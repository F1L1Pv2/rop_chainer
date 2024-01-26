use std::fs::File;
use std::env;
use std::process::exit;
use std::io::Read;
use std::collections::HashMap;

fn err(s: &str) {
    println!("Error: {}", s);
    exit(1);
}

#[derive(Debug,Clone, PartialEq)]
enum Token{
    Ident(String),
    Address(String),
    Text(String),
    Assign,
    NewLine
}

fn trim_start(s: String) -> String {
    s.trim_start_matches(|c: char| c.is_whitespace() && c != '\n').to_string()
}

impl Token{
    fn unwrap(&self, map: &HashMap<String, Vec<Token>>) -> Vec<Token>{
        if matches!(self, Token::Ident(_)){

            let val = match self{
                Token::Ident(val) => val,
                _ => ""
            };

            let mut tokens: Vec<Token> = Vec::new();
            
            let first_tokens: Vec<Token> = match map.get(val){
                Some(inner) => {
                    inner.clone()
                },
                None => {
                    err(&format!("{} doesn't exist", val));
                    Vec::new()
                }
            };

            for token in first_tokens{
                let mut inner_tokens = token.unwrap(map);
                tokens.append(&mut inner_tokens);
            }

            return tokens;
        }

        if matches!(self, Token::Address(_)){

            let mut val = match self{
                Token::Address(val) => val.to_string(),
                _ => "".to_string(),
            };

            let mut bytes: Vec<String> = Vec::new();

            val.remove(0);
            val.remove(0);

            if val.len() % 2 == 1{
                val.insert(0,'0');
            }

            while val.len()>0{
                let mut byte: String = val.remove(0).to_string();
                byte += &val.remove(0).to_string();
                bytes.push(byte.to_string());
            }

            bytes.reverse();

            let mut out = String::new();

            for byte in bytes{
                out += "\\x";
                out += &byte;
            }

            return vec!(Token::Text(out));
        }

        if matches!(self, Token::Text(_)){
            return vec!(self.clone());
        }

        Vec::new()
    }
}

fn tokenize(content: String) -> Vec<Token>{

    let mut content = content;

    let mut tokens: Vec<Token> = Vec::new();

    while !content.is_empty(){
        content = trim_start(content);
        if content.is_empty(){break;}

        if content.starts_with("//"){
            while content.chars().nth(0).unwrap() != '\n'{
                content.remove(0);
            }
            continue;
        }

        if content.starts_with("\""){
            let mut text = String::new();
            content.remove(0);
            while content.chars().nth(0).unwrap() != '\"'{
                text += &content.remove(0).to_string();
            }
            content.remove(0);
            tokens.push(Token::Text(text));
            continue;
        }

        if content.starts_with("0x"){
            let mut address = String::new();
            while content.chars().nth(0).unwrap().is_alphanumeric(){
                address += &content.remove(0).to_string();
            }
            tokens.push(Token::Address(address));
            continue;
        }

        if content.starts_with("="){
            content.remove(0);
            tokens.push(Token::Assign);
            continue;
        }

        if content.starts_with("\n"){
            content.remove(0);
            tokens.push(Token::NewLine);
            continue;
        }

        if content.chars().nth(0).unwrap().is_alphanumeric(){
            let mut ident = String::new();
            while content.chars().nth(0).unwrap().is_alphanumeric(){
                ident += &content.remove(0).to_string();
                if content.is_empty(){
                    break;
                }
            }

            tokens.push(Token::Ident(ident));
            continue;
        }

        content.remove(0);
    }

    return tokens;
}

fn concat_all_text(tokens: Vec<Token>) -> String{
    let mut out_string: String = String::new();
    
    for token in tokens{
        if !matches!(token, Token::Text(_)){
            err("concat_all_text accepts only text tokens");
        }

        let val = match token{
            Token::Text(val)=>val,
            _=>"".to_string()
        };

        out_string += &val;
    }

    out_string
}

fn main() {

    let mut args = env::args();

    let _ = args.next().unwrap();

    if args.len() == 0 {
        err("Filename Wasnt provided");
    }

    let filename = args.next().unwrap();

    let mut file = File::open(filename).expect("couldnt open file");

    let mut content = String::new();

    file.read_to_string(&mut content).expect("couldnty read file");

    let mut tokens = tokenize(content);

    let mut varaibles: HashMap<String, Vec<Token>> = HashMap::new();


    while !tokens.is_empty(){
        let token = tokens.remove(0);
        if matches!(token, Token::Ident(_)){
            let mut ident: String = String::new();
            match token{
                Token::Ident(val) => ident=val,
                _ => assert!(false),
            }

            let equ = tokens.remove(0);
            if matches!(equ, Token::Assign){
                let mut expr: Vec<Token> = Vec::new();
                while !matches!(tokens[0].clone(), Token::NewLine){
                    expr.push(tokens.remove(0));
                }
                tokens.remove(0);
                varaibles.insert(ident, expr);
            }else{
                err("Expected '='")
            }
        }else{

            if matches!(token, Token::NewLine){
                continue;
            }

            err("Expected Identifier")
        }
    } 

    if !varaibles.contains_key("out") {
        err("Return varaible out wasnt provided");
    }

    let parsed_tokens = Token::Ident("out".to_string()).unwrap(&varaibles);

    println!("{}", concat_all_text(parsed_tokens));

}
