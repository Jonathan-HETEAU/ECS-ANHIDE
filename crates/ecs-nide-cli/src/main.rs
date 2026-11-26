//! Le binaire `ecs-nide`.

fn main() {
    let cwd = std::env::current_dir().expect("dossier courant inaccessible");
    let code = ecs_nide_cli::run(
        std::env::args(),
        &cwd,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
