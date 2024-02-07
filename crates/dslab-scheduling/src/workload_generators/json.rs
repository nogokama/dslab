pub struct JSONWorkloadGenerator {
    pub path: String,
}

impl JSONWorkloadGenerator {
    pub fn new(path: String) -> JSONWorkloadGenerator {
        JSONWorkloadGenerator { path }
    }
}
