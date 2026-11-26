//! Le vrai binaire `ecs-nide`, lancé comme le lancerait un utilisateur ou une intégration continue.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/space-shooter")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn ecs_nide(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ecs-nide"))
        .args(args)
        .output()
        .expect("lancement du binaire")
}

#[test]
fn mcp_009_le_binaire_valide_le_projet_d_exemple() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&example(), tmp.path());
    let output = ecs_nide(&["validate", "--project", tmp.path().to_str().unwrap()]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Aucun diagnostic."
    );
}

#[test]
fn mcp_011_le_binaire_echoue_sur_un_projet_invalide() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&example(), tmp.path());
    let movement = tmp.path().join("systems/movement/system.yaml");
    let text = fs::read_to_string(&movement).unwrap();
    fs::write(
        &movement,
        text.replace("  Position: write\n", "  Position: write\n  Mana: read\n"),
    )
    .unwrap();
    let output = ecs_nide(&["validate", "--project", tmp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("composant inconnu : Mana"));
}

#[test]
fn mcp_009_l_aide_liste_les_commandes() {
    let output = ecs_nide(&["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    for command in [
        "new",
        "validate",
        "plan",
        "contract",
        "gen-tests",
        "import-results",
    ] {
        assert!(
            help.contains(command),
            "{command} absent de l'aide :\n{help}"
        );
    }
}

#[test]
fn une_commande_inconnue_est_une_mauvaise_utilisation() {
    assert_eq!(ecs_nide(&["deploy"]).status.code(), Some(2));
}
