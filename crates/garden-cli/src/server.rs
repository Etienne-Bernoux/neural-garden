// Serveur HTTP minimal pour servir le viewer web et un fichier de montage JSON.

use std::fs;
use std::path::{Path, PathBuf};

use tiny_http::{Header, Response, Server};

/// Lance un serveur HTTP qui sert le viewer et un montage.
pub fn serve_replay(port: u16, montage_path: &Path, web_dir: &Path) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).map_err(|e| format!("Erreur serveur: {}", e))?;

    println!("Viewer disponible sur http://localhost:{}", port);
    println!("Ctrl+C pour arreter");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let url = url.split('?').next().unwrap_or(&url);

        let (content, content_type) = if url == "/" || url == "/index.html" {
            (read_file(&web_dir.join("index.html")), "text/html")
        } else if url == "/montage.json" {
            (read_file(montage_path), "application/json")
        } else if url == "/style.css" {
            match safe_path(web_dir, "style.css") {
                Some(p) => (read_file(&p), "text/css"),
                None => (None, "text/plain"),
            }
        } else if url.starts_with("/js/") {
            // Protection contre le directory traversal
            match safe_path(web_dir, &url[1..]) {
                Some(p) => (read_file(&p), "application/javascript"),
                None => (None, "text/plain"),
            }
        } else {
            (None, "text/plain")
        };

        let response = match content {
            Some(data) => {
                // Les arguments sont des literals connus — unwrap est sur ici
                let header = Header::from_bytes("Content-Type", content_type).unwrap();
                Response::from_data(data).with_header(header)
            }
            None => {
                if content_type == "text/plain" && url.starts_with("/js/") {
                    Response::from_string("403 Forbidden").with_status_code(403)
                } else {
                    Response::from_string("404 Not Found").with_status_code(404)
                }
            }
        };

        let _ = request.respond(response);
    }

    Ok(())
}

fn read_file(path: &Path) -> Option<Vec<u8>> {
    fs::read(path).ok()
}

/// Verifie qu'un chemin relatif ne sort pas du dossier web (protection directory traversal).
fn safe_path(web_dir: &Path, relative: &str) -> Option<PathBuf> {
    let path = web_dir.join(relative);
    let canonical = path.canonicalize().ok()?;
    let web_canonical = web_dir.canonicalize().ok()?;
    if canonical.starts_with(&web_canonical) {
        Some(canonical)
    } else {
        None
    }
}

/// Trouve le dossier web/ relatif au binaire ou au cwd.
pub fn find_web_dir() -> Option<PathBuf> {
    // Chercher dans le cwd d'abord
    let cwd_web = Path::new("web");
    if cwd_web.exists() {
        return Some(cwd_web.to_path_buf());
    }

    // Chercher relatif au binaire
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent()?;
        let exe_web = exe_dir.join("web");
        if exe_web.exists() {
            return Some(exe_web);
        }
        // Remonter dans l'arborescence du projet
        let project_web = exe_dir.join("../../web");
        if project_web.exists() {
            return Some(project_web);
        }
    }

    None
}
