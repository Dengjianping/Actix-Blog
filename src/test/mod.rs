pub(self) mod test_auth_views;
pub(self) mod test_post_views;

use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub(self) fn generate_random_string(length: usize) -> String {
    iter::repeat(()).map(|()| thread_rng().sample(Alphanumeric))
                    .take(length)
                    .collect::<String>()
}