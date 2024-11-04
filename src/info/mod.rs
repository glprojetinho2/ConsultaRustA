/*!
Extrai informações do website do ConsultaCA e as usa para popular o struct CA.
*/
// TODO: fazer a versão debug (erros existem) e release (erros não existem)
mod util;
use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
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
    ca: u32,
    laudo: Laudo,
    fabricante: Fabricante,
}
#[derive(Debug, PartialEq)]
struct Laudo {
    descricao: String,
    cnpj_laboratorio: u64,
    razao_social_laboratorio: String,
}
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

impl CA {
    /// Consulta a página do website do ConsultaCA e popula uma instância do struct CA.
    pub async fn consultar(ca: u32, client: reqwest::Client) -> Result<CA> {
        let resp = client
            .get("https://consultaca.com/".to_owned() + &ca.to_string())
            .send()
            .await;
        let body_txt = match resp {
            Ok(r) => match r.text().await {
                Ok(txt) => txt,
                Err(e) => panic!("{}", e),
            },
            Err(e) => panic!("{}", e),
        };

        // A extração da informação ocorre de duas formas:
        // através do body do site e através dum HashMap
        // gerado a partir dos parágrafos do site.
        let body = Html::parse_document(&body_txt);
        let p_info_hashmap = CA::paragrafos_hashmap(&body)?;

        Ok(CA {
            validade: CA::validade(&p_info_hashmap),
            processo: CA::processo(&p_info_hashmap),
            descricao: CA::descricao(&body),
            grupo: CA::grupo(&body),
            natureza: CA::natureza(&p_info_hashmap),
            situacao: CA::situacao(&p_info_hashmap),
            aprovado_para: CA::aprovado_para(&p_info_hashmap),
            cores: CA::cores(&p_info_hashmap),
            marcacao: CA::marcacao(&p_info_hashmap),
            referencias: CA::referencias(&p_info_hashmap),
            normas: CA::normas(&body),
            descricao_completa: CA::descricao_completa(&body),
            ca,
            laudo: Laudo {
                descricao: Laudo::descricao(&p_info_hashmap),
                cnpj_laboratorio: Laudo::cnpj_laboratorio(&p_info_hashmap),
                razao_social_laboratorio: Laudo::razao_social_laboratorio(&p_info_hashmap),
            },
            fabricante: Fabricante {
                razao_social: Fabricante::razao_social(&p_info_hashmap),
                cnpj: Fabricante::cnpj(&p_info_hashmap),
                nome_fantasia: Fabricante::nome_fantasia(&p_info_hashmap),
                cidade: Fabricante::cidade(&p_info_hashmap),
                uf: Fabricante::uf(&p_info_hashmap),
                qtd_cas: Fabricante::qtd_cas(&body),
                link: Fabricante::link(&body),
            },
        })
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
    fn paragrafos_hashmap(body: &Html) -> Result<HashMap<String, String>> {
        let selector = Selector::parse("p")?;
        let p_info = body.select(&selector);
        let mut resultado = HashMap::new();
        for paragrafo in p_info {
            let texto = paragrafo.text().collect::<String>();
            // [chave, valor]
            let par = texto.split(":").collect::<Vec<&str>>();
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
    /// assert_eq!(CA::extrair("chave", &info, |a| a.to_uppercase(), "".to_string()), "VALOR");
    /// ```
    fn extrair<T, F>(
        informacao: &str,
        hashmap: &HashMap<String, String>,
        parse_callback: F,
        padrao: T,
    ) -> T
    where
        F: Fn(String) -> T,
    {
        let result = match hashmap.get(informacao) {
            Some(value) => parse_callback(value.to_string()),
            None => return padrao,
        };
        result
    }

    fn validade(p_info: &HashMap<String, String>) -> NaiveDate {
        CA::extrair(
            "validade",
            p_info,
            |a| {
                // valor na forma `26/06/2029vencerá daqui 1699 dias`
                let validade_vec = &a[..10];
                match NaiveDate::parse_from_str(validade_vec, "%d/%m/%Y") {
                    Ok(v) => v,
                    Err(_) => NaiveDate::parse_from_str("01/01/0001", "%d/%m/%Y").unwrap(),
                }
            },
            NaiveDate::parse_from_str("01/01/0001", "%d/%m/%Y").unwrap(),
        )
    }
    fn grupo(body: &Html) -> String {
        let selector = Selector::parse(".grupo-epi-desc").unwrap();
        let grupo = match body.select(&selector).next() {
            Some(e) => e.text().collect::<String>(),
            None => return "".to_string(),
        };
        if grupo.is_empty() {
            return "".to_string();
        }
        grupo
    }
    fn descricao(body: &Html) -> String {
        // a descrição é o primeiro h1 da página
        let selector = Selector::parse("h1").unwrap();
        let descricao = match body.select(&selector).next() {
            Some(e) => e.text().collect::<String>(),
            None => return "".to_string(),
        };
        if descricao.is_empty() {
            return "".to_string();
        }
        descricao
    }
    fn normas(body: &Html) -> Vec<String> {
        let selector = Selector::parse(".lista-normas").unwrap();
        let normas = match body.select(&selector).next() {
            Some(e) => e.text().map(|x| x.to_string()).collect::<Vec<String>>(),
            None => return vec![],
        };
        if normas.is_empty() {
            return vec![];
        }
        normas
    }
    fn processo(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair(
            "n° processo",
            p_info,
            |a| a.trim().parse::<u64>().unwrap_or_default(), 
            0,
        )
    }
    fn natureza(p_info: &HashMap<String, String>) -> String {
        CA::extrair("natureza", p_info, |a| a, "".to_string())
    }
    fn situacao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("situação", p_info, |a| a, "".to_string())
    }
    fn cores(p_info: &HashMap<String, String>) -> Vec<String> {
        CA::extrair(
            "cor",
            p_info,
            |a| {
                let cores_vec = a
                    .split(", ")
                    .map(|x| x.trim().to_lowercase().replace(".", ""))
                    .collect::<Vec<String>>();
                cores_vec
            },
            vec![],
        )
    }
    fn marcacao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("marcação", p_info, |a| a, "".to_string())
    }
    fn referencias(p_info: &HashMap<String, String>) -> String {
        CA::extrair("referências", p_info, |a| a, "".to_string())
    }
    fn aprovado_para(p_info: &HashMap<String, String>) -> String {
        CA::extrair("aprovado para", p_info, |a| a, "".to_string())
    }
    fn descricao_completa(body: &Html) -> String {
        let nome_h3 = "descrição completa";
        let selector = match Selector::parse("h3") {
            Ok(v) => v,
            Err(_) => return "".to_string(),
        };
        let h3s = body.select(&selector);
        for h3 in h3s {
            if h3.text().collect::<String>().to_lowercase() == nome_h3 {
                let descr_node = match h3.next_sibling() {
                    Some(v) => v,
                    None => return "".to_string(),
                };
                let descricao = match ElementRef::wrap(descr_node) {
                    Some(v) => v.text().collect::<String>(),
                    None => return "".to_string(),
                };
                return descricao;
            }
        }
        "".to_string()
    }
}

impl Laudo {
    fn descricao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("n° do laudo", p_info, |a| a, "".to_string())
    }
    fn razao_social_laboratorio(p_info: &HashMap<String, String>) -> String {
        CA::extrair("razão social", p_info, |a| a, "".to_string())
    }

    fn cnpj_laboratorio(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair(
            "cnpj do laboratório",
            p_info,
            |a| {
                // quero somente os números do CNPJ
                a
                    .chars()
                    .filter(|x| x.is_numeric())
                    .collect::<String>()
                    .parse::<u64>().unwrap_or_default()
            },
            0,
        )
    }
}
impl Fabricante {
    //TODO: colocar CA nos erros para facilitar debugging.
    fn link(body: &Html) -> String {
        let padrao = "".to_string();
        let selector = match Selector::parse("[href*=\"https://consultaca.com/fabricantes/\"]") {
            Ok(v) => v,
            Err(_) => return padrao,
        };
        let selected_iter = body.select(&selector);
        let a_element = match selected_iter.into_iter().next() {
            Some(v) => v,
            None => return padrao,
        };
        match a_element.attr("href") {
            Some(v) => v.to_string(),
            None => padrao,
        }
    }
    fn razao_social(p_info: &HashMap<String, String>) -> String {
        CA::extrair("razão social", p_info, |a| a, "".to_string())
    }
    fn cnpj(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair(
            "cnpj",
            p_info,
            |a| {
                // quero somente os números do CNPJ
                a
                    .chars()
                    .filter(|x| x.is_numeric())
                    .collect::<String>()
                    .parse::<u64>().unwrap_or_default()

            },
            0,
        )
    }
    fn nome_fantasia(p_info: &HashMap<String, String>) -> String {
        CA::extrair("nome fantasia", p_info, |a| a, "".to_string())
    }
    fn cidade(p_info: &HashMap<String, String>) -> String {
        let padrao = "".to_string();
        let cidade_uf_str = CA::extrair("cidade/uf", p_info, |a| a, "".to_string());
        if cidade_uf_str.is_empty() {
            return "".to_string();
        }
        let cidade_uf_vec = cidade_uf_str.split("/").collect::<Vec<&str>>();
        // se par existe, então ...
        if cidade_uf_vec.len() == 2 {
            cidade_uf_vec[0].to_string()
        } else {
            padrao
        }
    }
    fn uf(p_info: &HashMap<String, String>) -> String {
        let padrao = "".to_string();
        // we dont care about DRY around here
        let cidade_uf_str = CA::extrair("cidade/uf", p_info, |a| a, "".to_string());
        if cidade_uf_str.is_empty() {
            return "".to_string();
        }
        let cidade_uf_vec = cidade_uf_str.split("/").collect::<Vec<&str>>();
        // se par existe, então ...
        if cidade_uf_vec.len() == 2 {
            cidade_uf_vec[1].to_string()
        } else {
            padrao
        }
    }
    fn qtd_cas(body: &Html) -> u16 {
        let padrao = 0;
        let selector = Selector::parse(".total.info.load-blockui").unwrap();
        let result = match body.select(&selector).next() {
            Some(e) => match e.text().collect::<String>().parse::<u16>() {
                Ok(v) => v,
                Err(_) => return padrao,
            },
            None => return padrao,
        };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extrair() {
        let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
        assert_eq!(
            CA::extrair("chave", &info, |a| a.to_string(), "".to_string()),
            "valor"
        );
    }

    #[test]
    fn test_extrair_sem_key_no_hashmap() {
        let info = HashMap::from([]);
        assert_eq!(
            CA::extrair("chave", &info, |a| a.to_string(), "".to_string()),
            ""
        );
    }

    #[test]
    fn test_extrair_parsing() {
        let info = HashMap::from([("chave".to_string(), "valor".to_string())]);
        assert_eq!(
            CA::extrair("chave", &info, |a| a.to_uppercase(), "".to_string()),
            "VALOR"
        );
    }

    #[test]
    fn test_paragrafos_hashmap() {
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
        let resultado = CA::paragrafos_hashmap(&documento).expect("criação do hashmap falhou.");
        assert_eq!(
            resultado,
            HashMap::from([
                ("n° processo".to_string(), "19980274164202499".to_string()),
                ("situação".to_string(), "VENCIDO".to_string())
            ])
        );
    }
    #[tokio::test]
    async fn test_consultar() {
        let ca_codigo = 32551;
        let client = reqwest::Client::new();
        let ca = match CA::consultar(ca_codigo, client).await {
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
        descricao: "85.858; 87.820; 87.821.".to_string().to_string(),
        cnpj_laboratorio: 63025530004282,
        razao_social_laboratorio: "SEÇÃO TÉCNICA DE DESENVOLVIMENTO TECNOLÓGICO EM SAÚDE - IEE/USP".to_string(),
    },
    fabricante: Fabricante {
        razao_social: "SEÇÃO TÉCNICA DE DESENVOLVIMENTO TECNOLÓGICO EM SAÚDE - IEE/USP".to_string(),
        cnpj: 177445000141,
        nome_fantasia: "FARP UNIFORMES".to_string(),
        cidade: "ITUMBIARA".to_string(),
        uf: "GO".to_string(),
        qtd_cas: 28,
        link: "".to_string(),
    }
};
        assert_eq!(ca,ca_esperado)
    }
}
