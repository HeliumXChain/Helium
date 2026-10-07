//! Q5: named job templates — one word to a ready-to-run job definition.
//!
//! A template only declares WHAT to run (image, GPUs, ports, command).
//! Pulling the image / booting it is the provider's job (Firecracker rootfs
//! today, `docker run` where Docker exists). Unknown names are rejected with
//! the known list so typos fail fast, locally and over HTTP.

#[derive(Debug, Clone, PartialEq)]
pub struct JobTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub docker_image: &'static str,
    pub gpus: u32,
    pub ports: &'static [u16],
    pub command: &'static str,
}

const JUPYTER_PORTS: &[u16] = &[8888];

/// The whole registry. Keep it short: every entry is a promise to users.
pub fn list() -> Vec<JobTemplate> {
    vec![
        JobTemplate {
            name: "jupyter",
            description: "JupyterLab with PyTorch (port 8888, no token)",
            docker_image: "jupyter/pytorch-notebook:latest",
            gpus: 1,
            ports: JUPYTER_PORTS,
            command: "start-notebook.sh --NotebookApp.token= --NotebookApp.ip=0.0.0.0",
        },
        JobTemplate {
            name: "train",
            description: "PyTorch training container (1 GPU, run your script)",
            docker_image: "pytorch/pytorch:latest",
            gpus: 1,
            ports: &[],
            command: "python train.py",
        },
        JobTemplate {
            name: "batch-scan",
            description: "CPU batch job on Ubuntu (Kuro scans, ETL, benchmarks)",
            docker_image: "ubuntu:22.04",
            gpus: 0,
            ports: &[],
            command: "bash run.sh",
        },
    ]
}

/// Case-insensitive lookup. Empty input = no template (caller decides).
pub fn resolve(name: &str) -> Option<JobTemplate> {
    let want = name.trim().to_lowercase();
    if want.is_empty() {
        return None;
    }
    list().into_iter().find(|t| t.name == want)
}

/// Known names for error messages and CLI help.
pub fn names() -> Vec<&'static str> {
    list().iter().map(|t| t.name).collect()
}

/// Merge explicit values over a template: explicit image/gpus win,
/// template fills the blanks. Unknown template name = Err with the list.
pub fn apply(template: &str, image: &str, gpus: u32) -> Result<(String, u32), String> {
    let template = template.trim();
    if template.is_empty() {
        return Ok((image.trim().to_string(), gpus));
    }
    match resolve(template) {
        None => Err(format!("unknown template '{}' (known: {})", template, names().join(", "))),
        Some(t) => {
            let img = if image.trim().is_empty() {
                t.docker_image.to_string()
            } else {
                image.trim().to_string()
            };
            Ok((img, if gpus == 0 { t.gpus } else { gpus }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_three() {
        let all = list();
        assert_eq!(all.len(), 3);
        assert!(all.iter().any(|t| t.name == "jupyter"));
    }

    #[test]
    fn resolve_is_forgiving() {
        assert_eq!(resolve("Jupyter").unwrap().docker_image, "jupyter/pytorch-notebook:latest");
        assert_eq!(resolve("  train ").unwrap().gpus, 1);
        assert!(resolve("").is_none());
        assert!(resolve("nope").is_none());
    }

    #[test]
    fn apply_prefers_explicit() {
        let (img, gpus) = apply("jupyter", "", 0).unwrap();
        assert_eq!((img.as_str(), gpus), ("jupyter/pytorch-notebook:latest", 1));
        let (img, gpus) = apply("jupyter", "custom:1", 4).unwrap();
        assert_eq!((img.as_str(), gpus), ("custom:1", 4));
        assert!(apply("nope", "", 0).is_err());
        let (img, gpus) = apply("", "ubuntu:22.04", 0).unwrap();
        assert_eq!((img.as_str(), gpus), ("ubuntu:22.04", 0));
    }
}
