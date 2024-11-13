# ConsultaRustA

Ferramenta de linha de comando que busca informações sobre CA's no site
consultaca.com.

## Contruir executável

Antes de tudo, instale o rust:

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

Depois execute os comandos abaixo:

```bash
git clone https://github.com/glprojetinho2/ConsultaRustA.git
cd ConsultaRustA/crates/cli
cargo build --release
./target/release/consultarca
```

## Uso

O comando é autoexplicativo (por enquanto). Veja o uso básico:

```bash
consultarca 445
```

```
CA {
    descricao: "RESPIRADOR PURIFICADOR DE AR TIPO PEÇA SEMIFACIAL FILTRANTE PARA PARTÍCULAS PFF1",
    grupo: "Proteção Respiratória",
    natureza: "Nacional",
    validade: 2025-02-16,
    descricao_completa: "Respirador purificador de ar tipo peça semifacial filtrante para partículas, classe PFF-1 (S), com formato tipo concha, tamanho regu
lar, com solda térmica em seu perímetro. Sobre a concha interna de sustentação em microfibras sintéticas moldadas a quente em processo sem uso de resina, é m
ontado o meio filtrante composto por camadas de microfibras sintéticas tratadas eletrostaticamente. A parte externa do respirador é recoberta por um não teci
do na cor branca, que protege o meio filtrante, evitando que as microfibras se soltem. Nas laterais de cada peça existem 04 (quatro) grampos metálicos, sendo
 dois de cada lado, por onde passam as pontas de 02 (dois) tirantes elásticos. A parte superior interna da peça possui uma tira de espuma na cor cinza, e a p
arte superior externa possui uma tira de material metálico moldável, ambos para ajuste nasal. \"ESTE EQUIPAMENTO DEVERÁ APRESENTAR O SELO DE MARCAÇÃO DO INME
TRO\".",
    situacao: "VÁLIDO",
    processo: 14022172116202139,
    aprovado_para: "PROTEÇÃO DAS VIAS RESPIRATÓRIAS DO USUÁRIO CONTRA POEIRAS E NÉVOAS (PFF1).",
    cores: [
        "branca",
    ],
    marcacao: "Na parte externa da concha ou elástico.",
    referencias: "3M 8720",
    normas: [
        "ABNT NBR 13698:2011",
    ],
    ca: 445,
    laudo: Laudo {
        descricao: "Certificado de Conformidade nº BR37289007",
        cnpj: 10000000000052,
        razao_social: "OCP: Bureau Veritas Certification - BVQI",
    },
    fabricante: Fabricante {
        razao_social: "3M DO BRASIL LTDA",
        cnpj: 45985371000108,
        nome_fantasia: "3M",
        cidade: "SUMARE",
        uf: "SP",
        qtd_cas: 367,
        link: "https://consultaca.com/fabricantes/selo/17/3m-do-brasil-ltda-3m",
    },
}
```

O programa foi testado no Ubuntu 22.04.1.

## API REST

Há também uma API no projeto. Para utilizá-la, execute os comandos abaixo:

```bash
git clone https://github.com/glprojetinho2/ConsultaRustA.git
cd ConsultaRustA/crates/api
cargo build --release
./target/release/api
```

O output é similar ao output mostrado na seção de uso.

## Testes

Os testes têm de ser executados com o comando `cargo test -- --test-threads=1`
ou [`cargo nextest run`](https://nexte.st/) para que os logs sejam testados
corretamente.
