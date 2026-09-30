//! Geracao de relatorios fisicos (HTML / TXT / JSON) — bug antigo: nunca gerados em disco.

use crate::app::messages::AuditData;

pub fn generate_json(audit: &AuditData) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "gerado_em": chrono::Local::now().to_rfc3339(),
        "auditoria": audit,
    }))
    .unwrap_or_else(|e| format!(r#"{{"erro":"{}"}}"#, e))
}

pub fn generate_txt(audit: &AuditData) -> String {
    let mut s = String::new();
    s.push_str("HIGHLANDER FORGE BLADE — RELATORIO DE AUDITORIA\n");
    s.push_str(&format!("Gerado em: {}\n\n", chrono::Local::now().format("%d/%m/%Y %H:%M:%S")));
    if let Some(c) = &audit.cpu {
        s.push_str(&format!("CPU: {} ({} cores / {} threads, {} MHz)\n", c.name, c.cores, c.threads, c.max_speed_mhz));
    }
    if let Some(m) = &audit.memory {
        s.push_str(&format!("RAM: {:.1} GB em {} modulo(s)\n", m.total_bytes as f64 / 1_073_741_824.0, m.modules.len()));
    }
    s.push_str("\nDISCOS:\n");
    for d in &audit.disks {
        s.push_str(&format!("  {} ({}) {} GB total / {:.1} GB livres ({:.1}% livre)\n",
            d.device_id, d.filesystem,
            d.total_bytes / 1_073_741_824,
            d.free_bytes as f64 / 1_073_741_824.0, d.percent_free));
    }
    if !audit.gpus.is_empty() {
        s.push_str("\nGPUS:\n");
        for g in &audit.gpus {
            s.push_str(&format!("  {} — driver {}\n", g.name, g.driver_version));
        }
    }
    if let Some(mb) = &audit.motherboard {
        s.push_str(&format!("\nPlaca-mae: {} {} (BIOS {} {})\n", mb.manufacturer, mb.product, mb.bios_vendor, mb.bios_version));
    }
    s.push_str(&format!("\nSERVICOS: {} (terceiros: {})\n", audit.services.len(),
        audit.services.iter().filter(|x| x.is_third_party).count()));
    s.push_str(&format!("PROGRAMAS INSTALADOS: {}\n", audit.software.len()));
    s.push_str(&format!("CHAVES DE INICIALIZACAO (Run): {}\n", audit.registry_run_keys.len()));
    s
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn generate_html(audit: &AuditData, machine_id: &str) -> String {
    let rows_disk: String = audit.disks.iter().map(|d| format!(
        "<tr><td>{}</td><td>{}</td><td>{:.1} GB</td><td>{:.1} GB</td><td>{:.1}%</td></tr>",
        esc(&d.device_id), esc(&d.filesystem),
        d.total_bytes as f64 / 1_073_741_824.0,
        d.free_bytes as f64 / 1_073_741_824.0, d.percent_free)).collect();
    let rows_svc: String = audit.services.iter().take(200).map(|s| format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
        esc(&s.name), esc(&s.display_name), esc(&s.state),
        if s.is_third_party { "Terceiro" } else { "Microsoft" })).collect();
    let rows_sw: String = audit.software.iter().take(500).map(|w| format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        esc(&w.display_name), esc(&w.display_version), esc(&w.publisher))).collect();

    format!(r#"<!DOCTYPE html>
<html lang="pt-BR"><head><meta charset="utf-8">
<title>HFB — Auditoria {machine_id}</title>
<style>
body{{font-family:Segoe UI,Arial,sans-serif;margin:2rem;color:#222}}
h1{{color:#0b5cad}} h2{{border-bottom:2px solid #0b5cad;padding-bottom:4px}}
table{{border-collapse:collapse;width:100%;margin-bottom:2rem}}
th,td{{border:1px solid #ccc;padding:6px 10px;text-align:left;font-size:14px}}
th{{background:#0b5cad;color:#fff}}
tr:nth-child(even){{background:#f4f8fb}}
.meta{{color:#666;font-size:13px}}
</style></head><body>
<h1>Highlander Forge Blade — Relatorio de Auditoria</h1>
<p class="meta">Maquina: {machine_id} · Gerado em: {ts} · Versao: {ver}</p>
{cpu}{ram}
<h2>Discos</h2>
<table><tr><th>Volume</th><th>FS</th><th>Total</th><th>Livre</th><th>% Livre</th></tr>{rows_disk}</table>
<h2>Servicos (amostra de 200)</h2>
<table><tr><th>Nome</th><th>Exibicao</th><th>Estado</th><th>Tipo</th></tr>{rows_svc}</table>
<h2>Programas instalados (amostra de 500)</h2>
<table><tr><th>Nome</th><th>Versao</th><th>Fabricante</th></tr>{rows_sw}</table>
</body></html>"#,
        machine_id = esc(machine_id),
        ts = chrono::Local::now().format("%d/%m/%Y %H:%M:%S"),
        ver = env!("CARGO_PKG_VERSION"),
        cpu = audit.cpu.as_ref().map(|c| format!(
            "<h2>CPU</h2><p>{} — {} cores / {} threads @ {} MHz ({})</p>",
            esc(&c.name), c.cores, c.threads, c.max_speed_mhz, esc(&c.architecture))).unwrap_or_default(),
        ram = audit.memory.as_ref().map(|m| format!(
            "<h2>Memoria</h2><p>{:.1} GB · {} modulo(s)</p>",
            m.total_bytes as f64 / 1_073_741_824.0, m.modules.len())).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::messages::{CpuInfo, DiskInfo};

    fn sample() -> AuditData {
        AuditData {
            cpu: Some(CpuInfo { name: "Intel <Test> & Co".into(), ..Default::default() }),
            disks: vec![DiskInfo { device_id: "C:".into(), total_bytes: 500 * 1_073_741_824, free_bytes: 100 * 1_073_741_824, percent_free: 20.0, ..Default::default() }],
            ..Default::default()
        }
    }

    #[test]
    fn json_report_is_valid_json() {
        let j = generate_json(&sample());
        serde_json::from_str::<serde_json::Value>(&j).expect("relatorio JSON deve ser valido");
    }

    #[test]
    fn txt_report_contains_sections() {
        let t = generate_txt(&sample());
        assert!(t.contains("CPU:"));
        assert!(t.contains("DISCOS:"));
    }

    #[test]
    fn html_escapes_injection() {
        let h = generate_html(&sample(), "id\"><script>alert(1)</script>");
        assert!(h.contains("&lt;script&gt;"), "entrada maliciosa deve ser escapada");
        assert!(!h.contains(r#""><script"#));
    }
}
