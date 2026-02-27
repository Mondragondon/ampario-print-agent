use std::process::Command;

/// Druckt ein PDF auf dem konfigurierten Drucker.
/// Nutzt plattformabhängig lpr (macOS/Linux) oder PowerShell (Windows).
pub fn print_pdf(
    pdf_path: &str,
    printer_name: &str,
    width_mm: u32,
    height_mm: u32,
) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        print_pdf_windows(pdf_path, printer_name)
    } else {
        print_pdf_unix(pdf_path, printer_name, width_mm, height_mm)
    }
}

/// macOS + Linux: Druckt via lpr (CUPS).
fn print_pdf_unix(
    pdf_path: &str,
    printer_name: &str,
    width_mm: u32,
    height_mm: u32,
) -> Result<(), String> {
    let page_size = format!("PageSize=Custom.{}x{}mm", width_mm, height_mm);

    let output = Command::new("lpr")
        .arg("-P")
        .arg(printer_name)
        .arg("-o")
        .arg(&page_size)
        .arg("-o")
        .arg("Resolution=300dpi")
        .arg("-o")
        .arg("fit-to-page")
        .arg("-o")
        .arg("page-left=0")
        .arg("-o")
        .arg("page-right=0")
        .arg("-o")
        .arg("page-top=0")
        .arg("-o")
        .arg("page-bottom=0")
        .arg(pdf_path)
        .output()
        .map_err(|e| format!("lpr Fehler: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Druckfehler: {}", stderr.trim()))
    }
}

/// Windows: Druckt via PowerShell Start-Process -Verb PrintTo.
#[allow(dead_code)]
fn print_pdf_windows(pdf_path: &str, printer_name: &str) -> Result<(), String> {
    // SumatraPDF (falls installiert) ist zuverlässiger für Zebra-Drucker
    let sumatra = std::path::Path::new(r"C:\Program Files\SumatraPDF\SumatraPDF.exe");
    if sumatra.exists() {
        let output = Command::new(sumatra)
            .arg("-print-to")
            .arg(printer_name)
            .arg("-silent")
            .arg("-print-settings")
            .arg("fit")
            .arg(pdf_path)
            .output()
            .map_err(|e| format!("SumatraPDF Fehler: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
        // Fallback zu PowerShell wenn SumatraPDF fehlschlägt
    }

    // Fallback: PowerShell Start-Process
    let ps_cmd = format!(
        "Start-Process -FilePath '{}' -Verb PrintTo -ArgumentList '\"{}\"' -Wait",
        pdf_path.replace('\'', "''"),
        printer_name.replace('\'', "''"),
    );

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&ps_cmd)
        .output()
        .map_err(|e| format!("PowerShell Fehler: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Druckfehler: {}", stderr.trim()))
    }
}

/// Listet alle verfügbaren Drucker.
/// macOS/Linux: lpstat -a | Windows: PowerShell Get-Printer
pub fn list_local_printers() -> Vec<String> {
    if cfg!(target_os = "windows") {
        list_printers_windows()
    } else {
        list_printers_unix()
    }
}

fn list_printers_unix() -> Vec<String> {
    match Command::new("lpstat").arg("-a").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    trimmed.split_whitespace().next().map(String::from)
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}

#[allow(dead_code)]
fn list_printers_windows() -> Vec<String> {
    let ps_cmd = "Get-Printer | Select-Object -ExpandProperty Name";
    match Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(ps_cmd)
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}
