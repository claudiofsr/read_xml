//! Regressão (Mandamento 03): o tempo decorrido (`-t`) deve ir para o STDERR.
//!
//! O stdout carrega apenas dados (chaves ausentes do `--filtro` com `-e`),
//! permitindo redirecionamento limpo: `read-xml ... -e > chaves_nao_encontradas`.

use std::fs;
use std::process::Command;

#[test]
fn test_elapsed_time_no_stderr_e_stdout_apenas_dados() {
    let dir = tempfile::tempdir().expect("criar diretório temporário");

    let xml_path = dir.path().join("entrada.xml");
    fs::write(
        &xml_path,
        b"<DocumentoDesconhecido><campo>1</campo></DocumentoDesconhecido>",
    )
    .unwrap();

    let chave_ausente = "35240147960950000121550020005910991037128568";
    let keys_path = dir.path().join("chaves.txt");
    fs::write(&keys_path, format!("{chave_ausente}\n")).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_read-xml"))
        .args(["-t", "-a", "-e", "-f"])
        .arg(&keys_path)
        .arg("-p")
        .arg(&xml_path)
        .current_dir(dir.path())
        .output()
        .expect("executar o binário read-xml");

    assert!(
        out.status.success(),
        "read-xml falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // stdout: somente a chave ausente (dado puro, redirecionável com `>`).
    assert_eq!(stdout.trim(), chave_ausente);
    assert!(
        !stdout.contains("Elapsed time"),
        "Elapsed time vazou para o stdout: {stdout:?}"
    );

    // stderr: diagnóstico e tempo visíveis no terminal mesmo com `> arquivo`.
    assert!(
        stderr.contains("Elapsed time:"),
        "Elapsed time ausente do stderr: {stderr:?}"
    );
    assert!(
        stderr.contains("Filtro de chaves:"),
        "relatório do filtro deve permanecer no stderr"
    );
}
