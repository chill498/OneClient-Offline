fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");

        let version = "2.9.0.4";
        res.set("CompanyName", "chill498 (community fork)");
        res.set("ProductName", "OneClient Offline");
        res.set("FileDescription", "OneClient Offline community launcher");
        res.set("LegalCopyright", "© 2026 Polyfrost Inc.");
        res.set("OriginalFilename", "oneclient_app.exe");
        res.set("InternalName", "oneclient_app");
        let numeric_version = (2u64 << 48) | (9u64 << 32) | 4;
        res.set_version_info(winresource::VersionInfo::FILEVERSION, numeric_version);
        res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric_version);
        res.set("FileVersion", version);
        res.set("ProductVersion", version);

        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
