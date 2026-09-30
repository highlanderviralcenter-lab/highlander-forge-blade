//! Build script — embute o manifest de elevacao (requireAdministrator) no binario Windows.
//!
//! Em plataformas nao-Windows, apenas registra a dependencia do arquivo para
//! invalidar o cache do build quando o manifest muda.

fn main() {
    println!("cargo:rerun-if-changed=assets/hfb.exe.manifest");
    println!("cargo:rerun-if-changed=assets/hfb.rc");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("assets/hfb.exe.manifest");
        if let Err(e) = res.compile() {
            println!("cargo:warning=falha ao embutir manifest: {}", e);
        }
    }
}
