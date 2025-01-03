use toml_edit::{DocumentMut, Table};

const LOCKFILE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));

pub struct Lockfile(DocumentMut);

impl Lockfile {
    pub fn new() -> Self {
        let lockfile = LOCKFILE.parse::<DocumentMut>().unwrap();
        println!("cargo:rerun-if-changed=Cargo.lock");

        Self(lockfile)
    }

    pub fn get_packages(&self) -> String {
        let packages = self.0.get("package").unwrap();
        let packages = packages.as_array_of_tables().unwrap();

        let mut items = vec![];
        for p in packages {
            let name = p.get("name").unwrap().as_str().unwrap();
            let version = p.get("version").unwrap().as_str().unwrap();

            let item = format!("(\"{name}\",\"{version}\")");
            items.push(item);
        }

        let length = items.len();
        let items = items.join(",");
        let items = format!("[{}]", items);
        format!("pub const PACKAGES: [(&str, &str); {length}] = {items};")
    }

    pub fn get_package(&self, name: &str) -> Option<&Table> {
        let packages = self.0.get("package")?.as_array_of_tables()?;

        packages.iter().find_map(|package| {
            let package_name = package.get("name")?.as_str()?;

            if package_name == name {
                Some(package)
            } else {
                None
            }
        })
    }
}
