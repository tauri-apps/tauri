fn main() {
    #[cfg(target_os = "windows")]
    embed_resource::compile("manifest.rc", embed_resource::NONE);
}
