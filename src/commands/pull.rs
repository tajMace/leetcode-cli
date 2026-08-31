// fetch a problem, write its starter code to src/problems/<slug>/<lang>.<fe>

use crate::error::Result;

pub fn pull(slug: String) -> Result<()> {
    // check if directory exists for slug; if so early return

    // create web client; if error, return error

    // pull question from slug; if error, return error

    // create directory and file -> normalise slug to snake_case; if error, return error

    // write header, supplied stub, empty fn main{} and empty testing block for additional tests; if error, return error

    Ok(())
}
