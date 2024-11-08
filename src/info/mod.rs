/*!
Extrai informações do website do ConsultaCA e as usa para popular o struct CA.
*/
mod util;
use chrono::NaiveDate;
use log::{error, warn};
use scraper::selectable::Selectable;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use util::extrair_numeros;
use util::Result;

/// Representa um CA.
/// A única coisa que o struct tem de saber é o código do CA.
/// O resto das informações será retirado do sítio https://consultaca.com/.
#[derive(Debug, PartialEq)]
pub struct CA {
    descricao: String,
    grupo: String,
    natureza: String,
    validade: chrono::NaiveDate,
    descricao_completa: String,
    situacao: String,
    processo: u64,
    aprovado_para: String,
    cores: Vec<String>,
    marcacao: String,
    referencias: String,
    normas: Vec<String>,
    pub ca: u32,
    laudo: Laudo,
    fabricante: Fabricante,
}
impl CA {
    /// Consulta a página do website do ConsultaCA e popula uma instância do struct CA.
    pub async fn consultar(body: &Html) -> Result<CA> {
        let p_info_hashmap = Extrator::paragrafos_hashmap(body)?;

        let ca = p_info_hashmap
            .get("n° ca")
            .unwrap_or(&"0".to_string())
            .parse()?;
        let extrator = Extrator::new(ca);

        let p_info_hashmap_fabricante = match extrator.secao_com_h3(body, "fabricante") {
            Some(v) => Extrator::paragrafos_hashmap(v)?,
            None => HashMap::new(),
        };
        let p_info_hashmap_laudo = match extrator.secao_com_h3(body, "laudos") {
            Some(v) => Extrator::paragrafos_hashmap(v)?,
            None => HashMap::new(),
        };

        Ok(CA {
            validade: extrator.validade(&p_info_hashmap),
            processo: extrator.processo(&p_info_hashmap),
            descricao: extrator.descricao(body),
            grupo: extrator.grupo(body),
            natureza: extrator.natureza(&p_info_hashmap),
            situacao: extrator.situacao(&p_info_hashmap),
            aprovado_para: extrator.aprovado_para(&p_info_hashmap),
            cores: extrator.cores(&p_info_hashmap),
            marcacao: extrator.marcacao(&p_info_hashmap),
            referencias: extrator.referencias(&p_info_hashmap),
            normas: extrator.normas(body),
            descricao_completa: extrator.descricao_completa(body),
            ca,
            laudo: Laudo::new(ca, &p_info_hashmap_laudo),
            fabricante: Fabricante::new(ca, &p_info_hashmap_fabricante, body),
        })
    }
}
/// Representa um laudo.
#[derive(Debug, PartialEq)]
struct Laudo {
    descricao: String,
    cnpj: u64,
    razao_social: String,
}
impl Laudo {
    fn new(ca: u32, p_info: &HashMap<String, String>) -> Self {
        let extrator = Extrator::new(ca);
        Laudo {
            descricao: extrator.descricao_laboratorio(p_info),
            razao_social: extrator.razao_social_laboratorio(p_info),
            cnpj: extrator.cnpj_laboratorio(p_info),
        }
    }
}

/// Representa um fabricante.
#[derive(Debug, PartialEq)]
struct Fabricante {
    razao_social: String,
    cnpj: u64,
    nome_fantasia: String,
    cidade: String,
    uf: String,
    qtd_cas: u16,
    link: String,
}
impl Fabricante {
    fn new(ca: u32, p_info: &HashMap<String, String>, body: &Html) -> Self {
        let extrator = Extrator::new(ca);
        Fabricante {
            cidade: extrator.cidade_fabricante(p_info),
            uf: extrator.uf_fabricante(p_info),
            razao_social: extrator.razao_social_fabricante(p_info),
            cnpj: extrator.cnpj_fabricante(p_info),
            nome_fantasia: extrator.nome_fantasia_fabricante(p_info),
            qtd_cas: extrator.qtd_cas_fabricante(body),
            link: extrator.link_fabricante(body),
        }
    }
}

/// Extrai dados da página do CA.
struct Extrator {
    ca: u32,
}

impl Extrator {
    fn new(ca: u32) -> Self {
        Extrator { ca }
    }
    /// Extrai informação específica duma lista de parágrafos da classe info.
    /// Esses parágrafos contém texto na forma chave:valor.
    /// Exemplo de parágrafo:
    /// ```html
    /// <p>
    ///     <strong>N° Processo:</strong>
    ///     <br>
    ///     19980274164202499
    /// </p>
    /// ```
    /// Se a chave ("N° Processo", no caso acima) contiver `chave`, então esse
    /// valor será retornado depois de passar pelo `parse_callback`.
    /// O argumento `nome` só serve para deixar mais claro o erro que ocorre
    /// quando não achamos a chave nos `p.info` (ou quando o valor é vazio).
    fn paragrafos_hashmap<'a, S: Selectable<'a> + Clone>(
        body: S,
    ) -> Result<HashMap<String, String>> {
        let selector = Selector::parse("p")?;
        let p_info = body.clone().select(&selector);
        let mut resultado = HashMap::new();
        for paragrafo in p_info {
            let texto = paragrafo.text().collect::<String>();
            let separator = "efa3fe20-aa7d-4672-be5a-890c505c3637";
            let chave_separada_do_valor = texto.replacen(":", separator, 1);
            // [chave, valor]
            let par = chave_separada_do_valor
                .split(separator)
                .collect::<Vec<&str>>();
            if par.len() == 2 && !&par[1].is_empty() {
                resultado.insert(par[0].trim().to_lowercase(), par[1].trim().to_string());
            }
        }
        Ok(resultado)
    }

    /// Retorna valor do `hashmap` associado à chave `informacao` depois
    /// de ter sido processado pela função `parse_callback`.
    /// Se o hashmap não tiver a chave `informacao`, então a função
    /// retorna o valor do argumento `padrao`.
    /// # Exemplo
    /// ```rust
    /// let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
    /// assert_eq!(self.extrair("chave", &info, |a| a.to_uppercase(), "".to_string()), "VALOR");
    /// ```
    fn extrair<T, F>(
        &self,
        informacao: &str,
        hashmap: &HashMap<String, String>,
        parse_callback: F,
        padrao: T,
    ) -> T
    where
        F: Fn(String) -> Result<T>,
    {
        let result = match hashmap.get(informacao) {
            Some(value) => match parse_callback(value.to_string()) {
                Ok(value) => value,
                Err(e) => {
                    return {
                        error!(
                            "CA{} {informacao}: erro no parsing. Veja: {:#?}",
                            self.ca, e
                        );
                        padrao
                    }
                }
            },
            None => {
                return {
                    warn!(
                        "CA{}: chave '{informacao}' não está presente no hashmap.",
                        self.ca
                    );
                    padrao
                }
            }
        };
        result
    }

    /// Extrai texto de dentro de um elemento do HTML da página.
    /// O fazemos partindo do primeiro elemento que corresponde ao
    /// `seletor`.
    fn so_com_seletor(&self, body: &Html, seletor: &str) -> String {
        let selector = Selector::parse(seletor).unwrap();
        let elemento_txt = match body.select(&selector).next() {
            Some(e) => e.text().collect::<String>(),
            None => {
                return {
                    warn!("CA{}: {seletor} não encontrado no iterator.", self.ca);
                    "".to_string()
                }
            }
        };
        if elemento_txt.is_empty() {
            return {
                warn!("CA{}: {seletor} encontrado, mas está sem texto.", self.ca);
                "".to_string()
            };
        }
        elemento_txt
    }
    /// Retorna um elemento HTML (selecionável) com base no seu h3 interno.
    /// # Exemplo de HTML
    /// ```html
    /// <div class="grupo_result_ca"> <!-- Esse é o elemento retornado -->
    ///   <h3>Nome do h3</h3> <!-- nome do h3 (pode ser maiúsculo ou minúsculo) -->
    ///   <p class="info">info</p>
    /// </div>
    /// ```
    fn secao_com_h3<'a>(&self, body: &'a Html, nome: &str) -> Option<ElementRef<'a>> {
        let selector = Selector::parse("h3").unwrap();
        let h3s = body.select(&selector);
        for h3 in h3s {
            if h3.text().collect::<String>().to_lowercase() == nome {
                let pai = match h3.parent() {
                    Some(v) => v,
                    None => {
                        return {
                            warn!("CA{}: pai do '{nome}' não encontrado.", self.ca);
                            None
                        }
                    }
                };
                // ElementRef implementa o trait `Selectable`
                return match ElementRef::wrap(pai) {
                    Some(v) => Some(v),
                    None => {
                        return {
                            warn!("CA{}: pai do '{nome}' não é um Node::Element.", self.ca);
                            None
                        }
                    }
                };
            };
        }

        None
    }

    fn validade(&self, p_info: &HashMap<String, String>) -> NaiveDate {
        self.extrair(
            "validade",
            p_info,
            |a| {
                // valor na forma `26/06/2029vencerá daqui 1699 dias`
                let validade_vec = &a[..10];
                Ok(NaiveDate::parse_from_str(validade_vec, "%d/%m/%Y")?)
            },
            NaiveDate::parse_from_str("01/01/0001", "%d/%m/%Y").unwrap(),
        )
    }
    fn grupo(&self, body: &Html) -> String {
        self.so_com_seletor(body, ".grupo-epi-desc")
    }
    fn descricao(&self, body: &Html) -> String {
        self.so_com_seletor(body, "h1")
    }
    fn normas(&self, body: &Html) -> Vec<String> {
        let selector = Selector::parse(".lista-normas").unwrap();
        let normas = match body.select(&selector).next() {
            Some(e) => e.text().map(|x| x.to_string()).collect::<Vec<String>>(),
            None => {
                return {
                    warn!("CA{}: norma não encontrada no iterator.", self.ca);
                    vec![]
                }
            }
        };
        if normas.is_empty() {
            return {
                warn!("CA{}: normas encontradas, mas não têm conteúdo.", self.ca);
                vec![]
            };
        }
        normas
    }
    fn processo(&self, p_info: &HashMap<String, String>) -> u64 {
        self.extrair("n° processo", p_info, |a| Ok(a.trim().parse::<u64>()?), 0)
    }
    fn natureza(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("natureza", p_info, Ok, "".to_string())
    }
    fn situacao(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("situação", p_info, Ok, "".to_string())
    }
    fn cores(&self, p_info: &HashMap<String, String>) -> Vec<String> {
        self.extrair(
            "cor",
            p_info,
            |a| {
                let cores_vec = a
                    .split(", ")
                    .map(|x| x.trim().to_lowercase().replace(".", ""))
                    .collect::<Vec<String>>();
                Ok(cores_vec)
            },
            vec![],
        )
    }
    fn marcacao(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("marcação", p_info, Ok, "".to_string())
    }
    fn referencias(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("referências", p_info, Ok, "".to_string())
    }
    fn aprovado_para(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("aprovado para", p_info, Ok, "".to_string())
    }
    fn descricao_completa(&self, body: &Html) -> String {
        let nome_h3 = "descrição completa";
        let p_selector = Selector::parse("p").unwrap();
        let elemento_descr = match self.secao_com_h3(body, nome_h3) {
            Some(v) => v,
            None => return "".to_string(),
        };
        let p = match elemento_descr.select(&p_selector).next() {
            Some(v) => v,
            None => return "".to_string(),
        };
        p.text().collect::<String>()
    }
    fn descricao_laboratorio(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("n° do laudo", p_info, Ok, "".to_string())
    }
    fn razao_social_laboratorio(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("razão social", p_info, Ok, "".to_string())
    }

    fn cnpj_laboratorio(&self, p_info: &HashMap<String, String>) -> u64 {
        self.extrair("cnpj do laboratório", p_info, extrair_numeros, 0)
    }
    fn razao_social_fabricante(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("razão social", p_info, Ok, "".to_string())
    }
    fn cnpj_fabricante(&self, p_info: &HashMap<String, String>) -> u64 {
        self.extrair("cnpj", p_info, extrair_numeros, 0)
    }
    fn nome_fantasia_fabricante(&self, p_info: &HashMap<String, String>) -> String {
        self.extrair("nome fantasia", p_info, Ok, "".to_string())
    }
    /// Retorna um par na forma (cidade, UF).
    fn cidade_uf_extrator(&self, p_info: &HashMap<String, String>) -> (String, String) {
        let padrao = "".to_string();
        let cidade_uf_str = self.extrair("cidade/uf", p_info, Ok, "".to_string());
        if cidade_uf_str.is_empty() {
            warn!("CA{}: par cidade/uf não encontrado (fabricante).", self.ca);
            return (padrao.to_owned(), padrao);
        }
        let cidade_uf_vec = cidade_uf_str.split("/").collect::<Vec<&str>>();
        // se par existe, então ...
        if cidade_uf_vec.len() == 2 {
            (cidade_uf_vec[0].to_string(), cidade_uf_vec[1].to_string())
        } else {
            warn!("CA{}: par cidade/uf não é um par (fabricante).", self.ca);
            (padrao.to_owned(), padrao)
        }
    }
    fn cidade_fabricante(&self, p_info: &HashMap<String, String>) -> String {
        self.cidade_uf_extrator(p_info).0
    }
    fn uf_fabricante(&self, p_info: &HashMap<String, String>) -> String {
        self.cidade_uf_extrator(p_info).1
    }
    fn qtd_cas_fabricante(&self, body: &Html) -> u16 {
        let padrao = 0;
        let selector = Selector::parse(".total.info.load-blockui").unwrap();
        let result = match body.select(&selector).next() {
            Some(e) => match e.text().collect::<String>().parse::<u16>() {
                Ok(v) => v,
                Err(_) => {
                    return {
                        warn!(
                            "CA{}: quantidade de CA's do fabricante não é um inteiro.",
                            self.ca
                        );
                        padrao
                    }
                }
            },
            None => {
                return {
                    warn!(
                        "CA{}: quantidade de CA's do fabricante não encontrada.",
                        self.ca
                    );
                    padrao
                }
            }
        };
        result
    }
    fn link_fabricante(&self, body: &Html) -> String {
        let padrao = "".to_string();
        let selector = Selector::parse("[href*=\"https://consultaca.com/fabricantes/\"]").unwrap();
        let selected_iter = body.select(&selector);
        let a_element = match selected_iter.into_iter().next() {
            Some(v) => v,
            None => {
                return {
                    warn!("CA{}: link do fabricante não encontrado.", self.ca);
                    padrao
                }
            }
        };
        match a_element.attr("href") {
            Some(v) => v.to_string(),
            None => {
                warn!(
                    "CA{}: elemento 'a' encontrado, mas não contém link (fabricante).",
                    self.ca
                );
                padrao
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use log4rs::{
        append::{console::ConsoleAppender, file::FileAppender},
        config::{Appender, Root},
        encode::pattern::PatternEncoder,
        Config,
    };
    use std::{
        fs::{self},
        sync::{LazyLock, Mutex},
    };

    static HANDLE: LazyLock<Mutex<log4rs::Handle>> = LazyLock::new(|| Mutex::new(setup_log()));

    /// Retorna um `Handle` que será usado para mudar
    /// as configurações do logger padrão.
    #[allow(unused_must_use)]
    fn setup_log() -> log4rs::Handle {
        let default = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build();

        let config = Config::builder()
            .appender(Appender::builder().build("default", Box::new(default)))
            .build(Root::builder().appender("default").build(LevelFilter::Warn))
            .unwrap();

        log4rs::init_config(config).unwrap()
    }

    /// Cria uma configuração para o logger e retorna um id.
    /// O logger padrão passará a escrever no arquivo `/tmp/{test_id}.log`.
    /// Cada teste que usa logging deve chamar essa função.
    /// Essa função não é o suficiente para isolar os logs de cada teste.
    /// Nós temos de rodar cada teste num processo separado para que o handle
    /// não seja alterado quando não deve ser alterado.
    /// (veja [`esse comentário`](https://github.com/rust-lang/rust/issues/47506#issuecomment-1655503393)).
    fn config_specific_test(test_id: &str) -> String {
        let encoder_str = "{d} - {m}{n}";
        let requests = FileAppender::builder()
            .append(false)
            .encoder(Box::new(PatternEncoder::new(encoder_str)))
            .build(format!("/tmp/{test_id}.log"))
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("requests", Box::new(requests)))
            .build(
                Root::builder()
                    .appender("requests")
                    .build(LevelFilter::Warn),
            )
            .unwrap();
        HANDLE.lock().unwrap().set_config(config);
        test_id.to_string()
    }

    /// Lê o conteúdo do log de um teste (veja `config_specific_test`).
    fn read_test(test_id: String) -> String {
        fs::read_to_string(format!("/tmp/{test_id}.log")).unwrap()
    }

    use super::*;
    #[test]
    fn extrair() {
        let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
        assert_eq!(
            Extrator::new(777).extrair("chave", &info, Ok, "".to_string()),
            "valor"
        );
    }

    #[test]
    fn extrair_sem_key_no_hashmap() {
        let test_id = config_specific_test("extrair_sem_key_no_hashmap");
        let info = HashMap::from([]);
        assert_eq!(
            Extrator::new(777).extrair("SoliDeoGloria", &info, Ok, "".to_string()),
            ""
        );
        let logs = read_test(test_id);
        println!("{}", logs);
        assert!(logs.contains("CA777: chave 'SoliDeoGloria'"));
    }

    #[test]
    fn extrair_parsing() {
        let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
        assert_eq!(
            Extrator::new(777).extrair("chave", &info, |a| Ok(a.to_uppercase()), "".to_string()),
            "VALOR"
        );
    }

    #[test]
    fn paragrafos_hashmap() {
        let html = r#"
    <!DOCTYPE html>
    <meta charset="utf-8">
     <p>
         <strong>N° Processo:</strong>
         <br>
         19980274164202499
     </p>
     <p><strong>Situação:</strong><br><span style="color: rgb(255, 0, 0); font-weight: bold; --darkreader-inline-color: #ff1a1a;" data-darkreader-inline-color="">VENCIDO</span></p>
"#;

        let documento = Html::parse_document(html);
        let resultado =
            Extrator::paragrafos_hashmap(&documento).expect("criação do hashmap falhou.");
        assert_eq!(
            resultado,
            HashMap::from([
                ("n° processo".to_string(), "19980274164202499".to_string()),
                ("situação".to_string(), "VENCIDO".to_string())
            ])
        );
    }
    #[test]
    fn erro_no_parsing() {
        let test_id = config_specific_test("erro_no_parsing");
        let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
        Extrator::new(777).extrair(
            "chave",
            &info,
            |_| Err("IesusHominumSalvator".into()),
            "".to_string(),
        );
        let content = read_test(test_id);
        println!("{}", content);
        assert!(content.contains("CA777 chave: erro no parsing"));
    }
    #[tokio::test]
    async fn consultar() {
        let sucesso_pagina = fs::read_to_string("src/info/pagina/sucesso.html").unwrap();
        let body = Html::parse_document(&sucesso_pagina);
        let ca = match CA::consultar(&body).await {
            Ok(v) => v,
            Err(e) => panic!("erro na consulta: {:#?}", e),
        };
        let ca_esperado = CA {
    descricao: "CALÇA".to_string(),
    grupo: "Proteção dos Membros Inferiores".to_string(),
    natureza: "Nacional".to_string(),
    validade: NaiveDate::from_ymd_opt(2026,10, 8).unwrap(),
    descricao_completa: "Calça de segurança confeccionada em uma camada de tecido Uniforte Pro FR, composto por 100% de algodão, fabricado pela empresa Companhia de Tecidos Santanense, com gramatura nominal de 7,66 oz/yd² (260 g/m²), ATPV 9,6 cal/cm².".to_string(),
    situacao: "VÁLIDO".to_string(),
    processo: 19980216122202352,
    aprovado_para: "PROTEÇÃO DAS PERNAS DO USUÁRIO CONTRA AGENTES TÉRMICOS PROVENIENTES DE ARCO ELÉTRICO E FOGO REPENTINO.".to_string(),
    cores: vec![],
    marcacao: "Na etiqueta".to_string(),
    referencias: "F23.16".to_string(),
    normas: vec![
        "ASTM D 6413:2015".to_string(),
        "ASTM F 1506-10a".to_string(),
        "ASTM F 1930:2018".to_string(),
        "ASTM F1959/F1959M-14".to_string(),
        "ASTM F2621-19".to_string(),
    ],
    ca: 32551,
    laudo: Laudo {
        descricao: "85.858; 87.820; 87.821.".to_string(),
        cnpj: 63025530004282,
        razao_social: "SEÇÃO TÉCNICA DE DESENVOLVIMENTO TECNOLÓGICO EM SAÚDE - IEE/USP".to_string(),
    },
    fabricante: Fabricante {
        razao_social: "FARP INDUSTRIA DE ROUPAS LTDA".to_string(),
        cnpj: 177445000141,
        nome_fantasia: "FARP UNIFORMES".to_string(),
        cidade: "ITUMBIARA".to_string(),
        uf: "GO".to_string(),
        qtd_cas: 28,
        link: "".to_string(),
    }
};
        assert_eq!(ca, ca_esperado)
    }

    #[tokio::test]
    async fn consultar_com_erro() {
        let test_id = config_specific_test("consultar_com_erro");
        let body = Html::parse_document("");
        let consulta = match CA::consultar(&body).await {
            Ok(v) => format!("{:#?}", v),
            Err(e) => panic!("erro na consulta: {:#?}", e),
        };
        let content = read_test(test_id);
        println!("{}", content);
        assert_eq!(
            content.matches("encontrad").count() + content.matches("presente").count(),
            // isto aqui é a quantidade de propriedades (que não são structs) do struct CA.
            // O -6 elimina as seguintes linhas:
            // CA {
            // laudo: Laudo {
            // },
            // fabricante: Fabricante {
            //     },
            // }
            consulta.lines().count() - 6
        );
    }
}
