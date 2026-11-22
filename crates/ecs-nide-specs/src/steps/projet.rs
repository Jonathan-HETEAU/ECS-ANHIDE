//! Étapes communes sur le projet.

use spec_markdown::Registry;

use crate::World;

pub fn register(r: &mut Registry<World>) {
    r.step("Ouvrir le projet d'exemple {}", |w, s| {
        w.open_example(s.param(0)?)
    });
}
