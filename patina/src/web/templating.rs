use miette::{IntoDiagnostic, Result};
use minijinja::{Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::path::PathBuf;
use std::sync::Arc;

// FIXME implement diagnostic into error handling, etc.
// FIXME implement IntoResponse directly

#[derive(Clone)]
pub(crate) struct Engine {
    inner: Arc<AutoReloader>,
}

impl Engine {
    pub fn new(template_path: PathBuf, disable_autoreload: bool) -> Self {
        Self {
            inner: Arc::new(AutoReloader::new(move |notifier| {
                let mut env = Environment::new();

                // if watch_path is never called, no fs watcher is created
                if !disable_autoreload {
                    notifier.watch_path(&template_path, true);
                }

                env.set_source(Source::from_path(&template_path));
                Ok(env)
            })),
        }
    }

    pub fn render<D: serde::Serialize>(&self, key: &str, data: D) -> Result<String> {
        let env = self.inner.acquire_env().into_diagnostic()?;
        let template = env.get_template(key).into_diagnostic()?;
        let rendered = template.render(&data).into_diagnostic()?;

        Ok(rendered)
    }
}
