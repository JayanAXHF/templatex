use crate::{
    errors::{Error, Result},
    filter::{Filter, FilterFn},
};
use derive_builder::Builder;
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};
use glob::glob;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tera::ast;
use tracing::{debug, info, warn};

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
        "pdf",
        "tmp",
        "bak",
    ])
});

pub static IMAGE_FILTER: LazyLock<Filter<&str>> = LazyLock::new(|| {
    Filter::<&str>::with_filter(vec![
        "png", "jpg", "jpeg", "gif", "svg", "bmp", "webp", "ico", "tif", "tiff", "eps", "ps", "ai",
        "raw", "xcf", "pdf",
    ])
});

#[derive(Debug, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Builder)]
#[builder(build_fn(skip))]
pub struct Engine {
    #[allow(dead_code)]
    #[builder(setter(into))]
    template_dirs: Vec<PathBuf>,
    #[builder(setter(into))]
    #[allow(dead_code)]
    exclude_filters: Option<Filter<String>>,
    #[builder(setter(into))]
    #[allow(dead_code)]
    include_filters: Option<Filter<String>>,
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
    image_files: Vec<PathBuf>,
}

#[derive(Debug, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Clone)]
#[getset(get_clone = "pub")]
pub struct TemplateFile {
    path: PathBuf,
    variables: Vec<String>,
}

impl EngineBuilder {
    pub fn build(self) -> Result<Engine> {
        let Some(template_dirs) = self.template_dirs else {
            return Err(Error::Other(color_eyre::eyre::eyre!(
                "No template directories provided"
            )));
        };
        let mut exclude_filter = self.exclude_filters.clone().unwrap_or_default();
        let exclude_filter = exclude_filter.as_mut();
        let mut include_filter = self.include_filters.clone().unwrap_or_default();
        let include_filter = include_filter.as_mut();
        let mut templates = Vec::new();
        for dir in template_dirs.clone() {
            let mut files = Vec::new();
            let glob = glob(&dir.join("**/*").display().to_string())?.filter_map(|e| e.ok());
            let mut image_files = Vec::new();
            let glob = glob.filter(|f| {
                let Some(ext) = f.extension() else {
                    return false;
                };
                let ext = ext.display().to_string();
                let ext = ext.as_str();
                if let Some(exclude) = &exclude_filter
                    && exclude.filter(f.display().to_string())
                {
                    return false;
                }
                if let Some(include) = &include_filter
                    && include.filter(f.display().to_string())
                {
                    return true;
                }
                if IMAGE_FILTER.filter(ext) {
                    image_files.push(f.clone());
                    return false;
                }

                !(FILE_FILTER.filter(ext) || IMAGE_FILTER.filter(ext))
            });
            let mut tera = tera::Tera::default();

            tera.add_template_files(glob.filter_map(|f| {
                let sf = f.strip_prefix(&dir).ok()?;
                let sf = sf.display().to_string();
                Some((f, Some(sf)))
            }))?;
            tera.build_inheritance_chains()?;
            tera.autoescape_on(vec![".tex"]);
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
                image_files,
            });
        }
        Ok(Engine {
            template_dirs,
            templates,
            exclude_filters: None,
            include_filters: None,
        })
    }
}

impl Engine {
    pub fn from_values(template_dirs: Vec<PathBuf>, templates: Vec<Template>) -> Self {
        Self {
            template_dirs,
            templates,
            exclude_filters: None,
            include_filters: None,
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
        let template_source_dir = template.dir();
        for f in template.image_files() {
            let output_file = out_dir
                .join("src")
                .join(f.strip_prefix(&template_source_dir).unwrap());
            info!("Rendering {}", output_file.display());
            let Some(prefix) = output_file.parent() else {
                warn!("Failed to get parent of {}", output_file.display());
                continue;
            };

            std::fs::create_dir_all(prefix)?;
            std::fs::create_dir_all(out_dir.join("src"))?;
            debug!(prefix = ?prefix.display(), out_dir = ?out_dir.display(),src_dir = ?out_dir.join("src").display(), "Creating directory");
            //std::fs::File::create(&output_file)?;
            std::fs::copy(f, output_file)?;
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
            debug!(prefix = ?prefix.display(), out_dir = ?out_dir.display(),src_dir = ?out_dir.join("src").display(), "Creating directory");
            let mut file = std::fs::File::create(out_dir.join("src").join(f))?;
            info!("Rendering {}", f);
            let render_result = template.tera.render(f, &context);
            debug!(render_result = ?render_result, "Rendered");
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
    pub config: LoadedTemplateDirConfig,
    pub dir: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoadedTemplateDirConfig {
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub ignore: bool,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

impl LoadedTemplateDir {
    pub fn new(name: String, description: Option<String>, dir: PathBuf) -> Self {
        let config = LoadedTemplateDirConfig {
            name,
            description,
            ignore: false,
            exclude: None,
            include: None,
        };
        Self { config, dir }
    }
    pub fn from_config(config: LoadedTemplateDirConfig, dir: PathBuf) -> Self {
        Self { config, dir }
    }
    pub fn name(&self) -> &str {
        self.config.name.as_str()
    }
    pub fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }
    pub fn dir(&self) -> &PathBuf {
        &self.dir
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
        debug!("{}", conf.display());
        debug!(exists = ?conf.exists(), "File exists");
        // if !conf.try_exists()? {
        //     warn!("No templatex.toml found in {}", self.display());
        //     let Some(dirname) = self.file_name() else {
        //         return Err(Error::IoError(io::ErrorKind::InvalidFilename.into()));
        //     };
        //     return Ok(LoadedTemplateDir::new(
        //         dirname.display().to_string(),
        //         None,
        //         self.to_path_buf(),
        //     ));
        // }
        let Ok(conf) = std::fs::read_to_string(conf) else {
            warn!("No templatex.toml found in {}", self.display());
            let Some(dirname) = self.file_name() else {
                return Err(Error::IoError(io::ErrorKind::InvalidFilename.into()));
            };
            return Ok(LoadedTemplateDir::new(
                dirname.display().to_string(),
                None,
                self.to_path_buf(),
            ));
        };
        debug!(conf);
        let conf: LoadedTemplateDirConfig = toml::from_str(&conf)?;
        Ok(LoadedTemplateDir::from_config(conf, self.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let _engine = EngineBuilder::default()
            .template_dirs(vec![PathBuf::from("./templates")])
            .clone()
            .build()
            .unwrap();
        // println!("{:#?}", engine.templates);
    }
}
