mod backend;
mod frontend;
mod models;

use backend::Backend;
use frontend::runfrontend;

fn main() -> anyhow::Result<()> {
    let mut backend = Backend::new();
    runfrontend(&mut backend)
}
