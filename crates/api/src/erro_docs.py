import re

descricao = "/// Erros que podem ocorrer ao chamar a API."
with open("src/errors.rs", "r") as f:
    content = f.read()
    erros = re.findall(r"\"(E\d\d\d\d:.+)\"", content)
    doc = "/// " + "\n/// ".join(erros)
    new_content = re.sub(
        descricao + r".*?\#\[macro_export\]",
        descricao + "\n" + doc + "\n#[macro_export]",
        content,
        1,
        re.DOTALL | re.MULTILINE,
    )
    print(new_content)
    with open("src/errors.rs", "w") as f:
        f.write(new_content)
