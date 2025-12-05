use crate::{
    errors::{Error, Result},
    filter::{Filter, FilterFn},
};
use derive_builder::Builder;
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};
use glob::glob;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tera::ast;
use tracing::{info, warn};

pub static FILE_FILTER: LazyLock<Filter<&str>> = LazyLock::new(|| {
    Filter::<&str>::with_filter(vec![
        "aux",
        "log",
        "out",
        "toc",
        "fls",
        "fdb_latexmk",
        "synctex.gz",
        "bbl",
        "blg",
        "run.xml",
        "nav",
        "snm",
        "vrb",
        "xdv",
        "pdf", // optional: remove this if you want PDFs included
        "tmp",
        "bak",
    ])
});

pub static IMAGE_FILTER: LazyLock<Filter<&str>> = LazyLock::new(|| {
    Filter::<&str>::with_filter(vec![
        "png", "jpg", "jpeg", "gif", "svg", "bmp", "webp", "ico", "tif", "tiff", "eps", "ps", "ai",
        "raw", "xcf",
    ])
});

#[derive(Debug, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Builder)]
#[builder(build_fn(skip, validate = "Self::validate"))]
pub struct Engine {
    #[builder(setter(into))]
    template_dirs: Vec<PathBuf>,
    #[builder(setter(skip))]
    templates: Vec<Template>,
}

#[derive(Debug, Getters, Setters, WithSetters, MutGetters, CloneGetters, Clone)]
#[getset(get_clone = "pub")]
pub struct Template {
    tera: tera::Tera,
    name: String,
    dir: PathBuf,
    files: Vec<TemplateFile>,
}

#[derive(Debug, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Clone)]
#[getset(get_clone = "pub")]
pub struct TemplateFile {
    path: PathBuf,
    variables: Vec<String>,
}

impl EngineBuilder {
    fn validate(&self) -> Result<()> {
        if self.template_dirs.is_none() {
            return Err(Error::Other(color_eyre::eyre::eyre!(
                "No template directories provided"
            )));
        }
        Ok(())
    }

    pub fn build(self) -> Result<Engine> {
        let Some(template_dirs) = self.template_dirs else {
            return Err(Error::Other(color_eyre::eyre::eyre!(
                "No template directories provided"
            )));
        };
        let mut templates = Vec::new();
        for dir in template_dirs.clone() {
            let mut files = Vec::new();
            let glob = glob(&dir.join("**/*").display().to_string())?.filter_map(|e| e.ok());
            let glob = glob.filter(|f| {
                let Some(ext) = f.extension() else {
                    return false;
                };
                let ext = ext.display().to_string();
                let ext = ext.as_str();
                !(FILE_FILTER.filter(ext) || IMAGE_FILTER.filter(ext))
            });
            let mut tera = tera::Tera::default();

            tera.add_template_files(glob.filter_map(|f| {
                let sf = f.strip_prefix(&dir).ok()?;
                let sf = sf.display().to_string();
                Some((f, Some(sf)))
            }))?;
            tera.build_inheritance_chains()?;
            info!("{:?}", tera);
            tera.autoescape_on(vec![".tex"]);
            info!(filter= ?*FILE_FILTER);
            for template in tera.get_template_names() {
                let mut variables = Vec::new();
                for node in tera.get_template(template).unwrap().ast.clone() {
                    if let ast::Node::VariableBlock(_, expr) = node
                        && let ast::ExprVal::Ident(s) = expr.val
                    {
                        variables.push(s);
                    }
                }
                files.push(TemplateFile {
                    variables,
                    path: dir.join(template),
                });
            }
            templates.push(Template {
                tera,
                name: dir.file_name().unwrap().to_str().unwrap().to_string(),
                dir,
                files,
            });
        }
        Ok(Engine {
            template_dirs,
            templates,
        })
    }
}

impl Engine {
    pub fn from_values(template_dirs: Vec<PathBuf>, templates: Vec<Template>) -> Self {
        Self {
            template_dirs,
            templates,
        }
    }
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.name == name)
    }
    pub fn render(&self, out_dir: &Path, name: &str, data: &[(String, String)]) -> Result<()> {
        let template = self
            .templates
            .iter()
            .find(|t| t.name == name)
            .ok_or(Error::Other(color_eyre::eyre::eyre!(
                "Template not found: {}",
                name
            )))?;
        let mut context = tera::Context::new();
        for (k, v) in data {
            context.insert(k, &v);
        }
        for f in template.tera.get_template_names() {
            let output_file = out_dir.join("src").join(f);
            info!("Rendering {}", output_file.display());
            let Some(prefix) = output_file.parent() else {
                warn!("Failed to get parent of {}", output_file.display());
                continue;
            };

            std::fs::create_dir_all(prefix)?;
            std::fs::create_dir_all(out_dir.join("src"))?;
            if FILE_FILTER.filter(f) {
                continue;
            }
            info!(prefix = ?prefix.display(), out_dir = ?out_dir.display(),src_dir = ?out_dir.join("src").display(), "Creating directory");
            let mut file = std::fs::File::create(out_dir.join("src").join(f))?;
            info!("Rendering {}", f);
            let render_result = template.tera.render(f, &context);
            info!(render_result = ?render_result, "Rendered");
            let Ok(rendered) = render_result else {
                tracing::error!(
                    "Failed to render template {}: {}",
                    f,
                    render_result.err().unwrap()
                );
                continue;
            };
            file.write_all(rendered.as_bytes())?;
        }
        let project_name = out_dir
            .file_name()
            .unwrap_or_default()
            .display()
            .to_string();
        let toml_str = include_str!("../tectonic_files/sample.toml");
        let mut temp_ctx = tera::Context::new();
        temp_ctx.insert("name", &project_name);
        let mut tera = template.tera.clone();
        let toml = tera.render_str(toml_str, &temp_ctx)?;

        fs::write(out_dir.join("Tectonic.toml"), toml)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LoadedTemplateDir {
    pub name: String,
    pub description: Option<String>,
    pub dir: PathBuf,
}

#[derive(Deserialize)]
pub struct LoadedTemplateDirConfig {
    pub name: String,
    pub description: Option<String>,
}

impl LoadedTemplateDir {
    pub fn new(name: String, description: Option<String>, dir: PathBuf) -> Self {
        Self {
            name,
            description,
            dir,
        }
    }
}

pub trait LoadableDir {
    fn load_dir(&self) -> Result<LoadedTemplateDir>;
}

impl LoadableDir for PathBuf {
    fn load_dir(&self) -> Result<LoadedTemplateDir> {
        if !self.is_dir() {
            return Err(Error::IoError(io::ErrorKind::NotADirectory.into()));
        }
        let conf = self.join("templatex.toml");
        if !conf.exists() {
            let Some(dirname) = self.file_name() else {
                return Err(Error::IoError(io::ErrorKind::InvalidFilename.into()));
            };
            return Ok(LoadedTemplateDir::new(
                dirname.display().to_string(),
                None,
                self.to_path_buf(),
            ));
        }
        let conf = std::fs::read_to_string(conf)?;
        let conf: LoadedTemplateDirConfig = toml::from_str(&conf)?;
        Ok(LoadedTemplateDir::new(
            conf.name,
            conf.description,
            self.to_path_buf(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use globset::{Glob, GlobBuilder, GlobSetBuilder};

    use super::*;
    #[test]
    fn test() {
        let engine = EngineBuilder::default()
            .template_dirs(vec![PathBuf::from("./templates")])
            .clone()
            .build()
            .unwrap();
        // println!("{:#?}", engine.templates);
    }
    #[test]
    fn test_glob() {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("**/*.!{tex}").unwrap());
        let set = builder.build().unwrap();
        let matches = set.matches_all("foo.tex");
        assert!(!matches);
    }
}
