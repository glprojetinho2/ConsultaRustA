/*!
Extrai informações do website do ConsultaCA e as usa para popular o struct CA.
*/
// TODO: testar
// TODO: fazer a versão debug (erros existem) e release (erros não existem)
mod util;
use chrono::{NaiveDate, NaiveTime};
use scraper::{ElementRef, Html, Selector};
use std::{collections::HashMap, str::FromStr};
use util::{nth_child, Result};


#[derive(Debug)]
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
#[derive(Debug)]
struct Laudo {
    descricao: String,
    cnpj_laboratorio: u64,
    razao_social_laboratorio: String,
}
#[derive(Debug)]
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
    /// Consulta a página do website do ConsultaCA e popula uma instância do struct CA
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
                resultado.insert(par[0].to_lowercase(), par[1].to_string());
            }
        }
        Ok(resultado)
    }

    fn extrair<T, F>(
        informacao: &str,
        hashmap: &HashMap<String, String>,
        parse_callback: F,
        padrao: T
    ) -> T
    where
        F: Fn(String) -> T,
    {
        let result = match hashmap.get(informacao) {
            Some(value) => parse_callback(value.to_string()),
            None => return padrao
        };
        result
    }
    pub fn validade(p_info: &HashMap<String, String>) -> NaiveDate {
        CA::extrair("validade", p_info, |a| {
            // valor na forma `26/06/2029vencerá daqui 1699 dias`
            let validade_vec = &a[..10];
            match NaiveDate::parse_from_str(validade_vec, "%d/%m/%Y") {
                Ok(v)=>v,
                Err(_)=>return NaiveDate::parse_from_str("01/01/0001", "%d/%m/%Y").unwrap()
            }
        }, NaiveDate::parse_from_str("01/01/0001", "%d/%m/%Y").unwrap())
    }
    pub fn grupo(body: &Html) -> String {
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
    pub fn descricao(body: &Html) -> String {
        // a descrição é o primeiro h1 da página
        let selector = Selector::parse("h1").unwrap();
        let descricao = match body.select(&selector).next() {
            Some(e) => e.text().collect::<String>(),
            None => return "".to_string()
        };
        if descricao.is_empty() {
return "".to_string();
        }
        descricao
    }
    pub fn normas(body: &Html) -> Vec<String> {
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
    pub fn processo(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair("n° processo", p_info, |a| match a.trim().parse::<u64>() {
            Ok(v)=>v,
            Err(_)=>return 0
        }, 0)
    }
    pub fn natureza(p_info: &HashMap<String, String>) -> String {
        CA::extrair("natureza", p_info, |a| a, "".to_string())
    }
    pub fn situacao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("situação", p_info, |a| a, "".to_string())
    }
    pub fn cores(p_info: &HashMap<String, String>) -> Vec<String> {
        CA::extrair("cor", p_info, |a| {
            let cores_vec = a
                .split(", ")
                .map(|x| x.trim().to_lowercase().replace(".", ""))
                .collect::<Vec<String>>();
            cores_vec
        }, vec![])
    }
    pub fn marcacao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("marcação", p_info, |a| a, "".to_string())
    }
    pub fn referencias(p_info: &HashMap<String, String>) -> String {
        CA::extrair("referências", p_info, |a| a, "".to_string())
    }
    pub fn aprovado_para(p_info: &HashMap<String, String>) -> String {
        CA::extrair("aprovado para", p_info, |a| a, "".to_string())
    }
    pub fn descricao_completa(body: &Html) -> String {
        let nome_h3 = "descrição completa";
        let selector = match Selector::parse("h3") {
            Ok(v) => v,
            Err(_) => return "".to_string()
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
    pub fn descricao(p_info: &HashMap<String, String>) -> String {
        CA::extrair("n° do laudo", p_info, |a| a, "".to_string())
    }
    pub fn razao_social_laboratorio(p_info: &HashMap<String, String>) -> String {
        CA::extrair("razão social", p_info, |a| a, "".to_string())
    }

    pub fn cnpj_laboratorio(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair("cnpj do laboratório", p_info, |a| {
            // quero somente os números do CNPJ
            match a.chars()
                .filter(|x| x.is_numeric())
                .collect::<String>()
                .parse::<u64>() {
                Ok(v)=>v,
                Err(_)=>return 0 }
        }, 0)
    }
}
impl Fabricante {
    //TODO: colocar CA nos erros para facilitar debugging.
    pub fn link(body: &Html) -> String {
        let padrao = "".to_string();
        let selector = match Selector::parse("[href*=\"https://consultaca.com/fabricantes/\"]") {
            Ok(v)=>v,
            Err(_)=>return padrao
        };
        let selected_iter = body.select(&selector);
        let a_element = match selected_iter.into_iter().next(){
            Some(v) => v,
            None => return padrao
        };
        match a_element.attr("href") {
            Some(v) => v.to_string(),
            None => return padrao
        }
    }
    pub fn razao_social(p_info: &HashMap<String, String>) -> String {
        CA::extrair("razão social", p_info, |a| a, "".to_string())
    }
    pub fn cnpj(p_info: &HashMap<String, String>) -> u64 {
        CA::extrair("cnpj", p_info, |a| {
            // quero somente os números do CNPJ
            match a.chars()
                .filter(|x| x.is_numeric())
                .collect::<String>()
                .parse::<u64>() {
                Ok(v)=>v,
                Err(_)=>return 0 
            }
        }, 0)
    }
    pub fn nome_fantasia(p_info: &HashMap<String, String>) -> String {
        CA::extrair("nome fantasia", p_info, |a| a, "".to_string())
    }
    pub fn cidade(p_info: &HashMap<String, String>) -> String {
        let padrao = "".to_string();
        let cidade_uf_str = CA::extrair("cidade/uf", p_info, |a| a, "".to_string());
        if cidade_uf_str.is_empty(){
            return "".to_string()
        }
        let cidade_uf_vec = cidade_uf_str.split("/").collect::<Vec<&str>>();
        // se par existe, então ...
        if (cidade_uf_vec.len()==2){
            cidade_uf_vec[0].to_string()
        } else {
            padrao
        }
    }
    pub fn uf(p_info: &HashMap<String, String>) -> String {
        let padrao = "".to_string();
        // we dont care about DRY around here
        let cidade_uf_str = CA::extrair("cidade/uf", p_info, |a| a,"".to_string());
        if cidade_uf_str.is_empty(){
            return "".to_string()
        }
        let cidade_uf_vec = cidade_uf_str.split("/").collect::<Vec<&str>>();
        // se par existe, então ...
        if (cidade_uf_vec.len()==2){
            cidade_uf_vec[1].to_string()
        } else {
            padrao
        }
    }
    pub fn qtd_cas(body: &Html) -> u16 {
        let padrao = 0;
        let selector = Selector::parse(".total.info.load-blockui").unwrap();
        let result = match body.select(&selector).next() {
            Some(e) => match e.text().collect::<String>().parse::<u16>() {
                Ok(v)=> v,
                Err(_) => return padrao
            },
            None => return padrao,
        };
        result
    }
}
