//! Rust port of [Ingreedy](https://github.com/iancanderson/ingreedy-js) - natural language parsing of recipe ingredients

#[macro_use]
extern crate pest_derive;

use lazy_static::lazy_static;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::ParseFloatError;
use thiserror::Error;

/// Ingreedy Error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum IngreedyError {
    /// Thrown if the wrong rule type is found as an inner pair of a given rule
    #[error("Wrong rule '{found:?}' for {rule:?}")]
    WrongRule {
        /// The errant child rule
        found: String,
        /// The parent rule
        rule: String,
    },
    /// Thrown if a given string could not be parsed as float
    #[error("Couldn't parse float")]
    ParseFloatError(#[from] ParseFloatError),
    /// Thrown if Pest fails to parse
    #[error("Pest failed to parse")]
    PestParseError(#[from] pest::error::Error<Rule>),
    /// Thrown if no inner rule found
    #[error("No inner rule found")]
    InnerRuleNoneError,
}

impl IngreedyError {
    /// Helper function to make the `WrongRule` error
    fn wrong_rule(found: &Pair<Rule>, rule: &str) -> Self {
        Self::WrongRule {
            found: found.as_str().into(),
            rule: rule.into(),
        }
    }
}

lazy_static! {
    static ref NUMBER_VALUE: HashMap<&'static str, f64> = {
        let mut map = HashMap::new();
        map.insert("a", 1.);
        map.insert("an", 1.);
        map.insert("zero", 0.);
        map.insert("one", 1.);
        map.insert("two", 2.);
        map.insert("three", 3.);
        map.insert("four", 4.);
        map.insert("five", 5.);
        map.insert("six", 6.);
        map.insert("seven", 7.);
        map.insert("eight", 8.);
        map.insert("nine", 9.);
        map.insert("ten", 10.);
        map.insert("eleven", 11.);
        map.insert("twelve", 12.);
        map.insert("thirteen", 13.);
        map.insert("fourteen", 14.);
        map.insert("fifteen", 15.);
        map.insert("sixteen", 16.);
        map.insert("seventeen", 17.);
        map.insert("eighteen", 18.);
        map.insert("nineteen", 19.);
        map.insert("twenty", 20.);
        map.insert("thirty", 30.);
        map.insert("forty", 40.);
        map.insert("fifty", 50.);
        map.insert("sixty", 60.);
        map.insert("seventy", 70.);
        map.insert("eighty", 80.);
        map.insert("ninety", 90.);
        map
    };
    static ref UNICODE_FRACTION_VALUE: HashMap<&'static str, f64> = {
        let mut map = HashMap::new();
        map.insert("¼", 1.0 / 4.);
        map.insert("½", 1.0 / 2.);
        map.insert("¾", 3.0 / 4.);
        map.insert("⅐", 1.0 / 7.);
        map.insert("⅑", 1.0 / 9.);
        map.insert("⅒", 1.0 / 10.);
        map.insert("⅓", 1.0 / 3.);
        map.insert("⅔", 2.0 / 3.);
        map.insert("⅕", 1.0 / 5.);
        map.insert("⅖", 2.0 / 5.);
        map.insert("⅗", 3.0 / 5.);
        map.insert("⅘", 4.0 / 5.);
        map.insert("⅙", 1.0 / 6.);
        map.insert("⅚", 5.0 / 6.);
        map.insert("⅛", 1.0 / 8.);
        map.insert("⅜", 3.0 / 8.);
        map.insert("⅝", 5.0 / 8.);
        map.insert("⅞", 7.0 / 8.);
        map
    };
}
#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
pub struct IngredientParser;

/// Ingredient information
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Ingredient {
    /// quantities for ingredient
    pub quantities: Vec<Quantity>,
    /// ingredient name
    pub ingredient: Option<String>,
}

/// System of unit used for a quantity
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnitType {
    English,
    Metric,
    Imprecise,
}

impl UnitType {
    fn parse(pair: &Pair<Rule>) -> Result<Self, IngreedyError> {
        match pair.as_rule() {
            Rule::imprecise_unit => Ok(Self::Imprecise),
            Rule::metric_unit => Ok(Self::Metric),
            Rule::english_unit => Ok(Self::English),
            _ => Err(IngreedyError::wrong_rule(pair, "unit_type")),
        }
    }
}

/// Quantity information
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Quantity {
    pub amount: f64,
    pub unit: Option<String>,
    pub unit_type: Option<UnitType>,
}

fn parse_multicharacter_fraction(fraction: &str) -> Result<f64, IngreedyError> {
    let numbers = fraction
        .split('/')
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(numbers[0] / numbers[1])
}

fn parse_fraction(pair: &Pair<Rule>) -> Result<f64, IngreedyError> {
    match pair.as_rule() {
        Rule::multicharacter_fraction => Ok(parse_multicharacter_fraction(pair.as_str())?),
        Rule::unicode_fraction => Ok(UNICODE_FRACTION_VALUE[pair.as_str()]),
        _ => Err(IngreedyError::wrong_rule(pair, "fraction")),
    }
}

fn parse_amount(pair: Pair<Rule>) -> Result<f64, IngreedyError> {
    match pair.as_rule() {
        Rule::float | Rule::integer => Ok(pair.as_str().parse()?),
        Rule::fraction => Ok(parse_fraction(&get_next_inner_pair(pair)?)?),
        Rule::mixed_number => Ok(pair
            .into_inner()
            .filter_map(|x| match x.as_rule() {
                Rule::integer => x.as_str().parse::<f64>().ok(),
                Rule::fraction => {
                    if let Ok(x) = get_next_inner_pair(x) {
                        parse_fraction(&x).ok()
                    } else {
                        None
                    }
                }
                Rule::separator => None,
                _ => panic!("wrong rule for mixed_number {:?}", x),
            })
            .sum()),
        Rule::number => Ok(NUMBER_VALUE[get_next_inner_pair(pair)?.as_str().trim()]),
        _ => Err(IngreedyError::wrong_rule(&pair, "amount")),
    }
}

impl Quantity {
    fn parse(pair: Pair<Rule>) -> Result<Self, IngreedyError> {
        let mut quantity = Self::default();
        match pair.as_rule() {
            Rule::amount_with_conversion | Rule::amount_with_attached_units => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::amount => {
                            quantity.amount = parse_amount(get_next_inner_pair(pair)?)?;
                        }
                        Rule::unit => {
                            let unit = get_next_inner_pair(pair)?;
                            quantity.unit_type = Some(UnitType::parse(&unit)?);
                            quantity.unit =
                                Some(format!("{:?}", get_next_inner_pair(unit)?.as_rule()));
                        }
                        _ => {}
                    }
                }
            }
            Rule::amount_with_multiplier => {
                let mut multiplier = 1.;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::amount => {
                            multiplier = parse_amount(get_next_inner_pair(pair)?)?;
                        }
                        Rule::parenthesized_quantity => {
                            let mut parenthesized_quantity = pair.into_inner();
                            parenthesized_quantity.next().unwrap();
                            quantity = Self::parse(parenthesized_quantity.next().unwrap())?;
                            quantity.amount *= multiplier;
                        }
                        _ => {}
                    }
                }
            }
            Rule::amount_imprecise => {
                let unit = get_next_inner_pair(pair)?;
                quantity.unit_type = Some(UnitType::parse(&unit)?);
                quantity.unit = Some(format!("{:?}", get_next_inner_pair(unit)?.as_rule()));
                quantity.amount = 1.;
            }
            _ => return Err(IngreedyError::wrong_rule(&pair, "quantity")),
        }

        Ok(quantity)
    }
}

fn get_next_inner_pair(pair: Pair<Rule>) -> Result<Pair<Rule>, IngreedyError> {
    pair.into_inner()
        .next()
        .ok_or(IngreedyError::InnerRuleNoneError)
}

impl Ingredient {
    /// Parse a single line of input into `Ingredient` information
    #[inline]
    pub fn parse(input: &str) -> Result<Self, IngreedyError> {
        Self::parse_pairs(IngredientParser::parse(Rule::ingredient_addition, input)?)
    }
    /// Parse `Ingredient` from Pest-returned Pairs<Rule> object
    #[inline]
    pub fn parse_pairs(pairs: Pairs<Rule>) -> Result<Self, IngreedyError> {
        let mut ingredient = Self {
            quantities: Vec::new(),
            ingredient: None,
        };
        for rule in pairs {
            match rule.as_rule() {
                Rule::multipart_quantity => {
                    for pair in rule.into_inner() {
                        if pair.as_rule() == Rule::quantity_fragment {
                            let quantity_fragment = get_next_inner_pair(pair)?;
                            let mut quantity = match quantity_fragment.as_rule() {
                                Rule::amount => Quantity {
                                    amount: parse_amount(get_next_inner_pair(quantity_fragment)?)?,
                                    ..Quantity::default()
                                },
                                Rule::quantity => {
                                    Quantity::parse(get_next_inner_pair(quantity_fragment)?)?
                                }
                                _ => {
                                    return Err(IngreedyError::wrong_rule(
                                        &quantity_fragment,
                                        "quantity_fragment",
                                    ))
                                }
                            };
                            if let Some(q) = ingredient.quantities.first() {
                                if q.unit.is_none() {
                                    quantity.amount *= q.amount;
                                    ingredient.quantities = Vec::new();
                                }
                            }
                            ingredient.quantities.push(quantity);
                        }
                    }
                }
                Rule::ingredient => {
                    let mut ing = rule.as_str();
                    if ing.starts_with("of ") {
                        ing = &ing[3..];
                    }
                    ingredient.ingredient = Some(ing.to_owned());
                }
                _ => {}
            }
        }
        Ok(ingredient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test1() {
        let input = "1.0 cup flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test2() {
        let input = "1 1/2 cups flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test3() {
        let input = "1 1/2 potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test4() {
        let input = "12345 potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 12345.);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test5() {
        let input = "1 2/3 cups flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 5. / 3.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test6() {
        let input = "12 (6-ounce) boneless skinless chicken breasts";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 72.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("boneless skinless chicken breasts".to_string())
        );
    }
    #[test]
    fn test7() {
        let input = "1 (28 ounce) can crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 28.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("can crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test8() {
        let input = "1/2 cups flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 0.5);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test9() {
        let input = "12g potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 12.);
        assert_eq!(ingredient.quantities[0].unit, Some("gram".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test10() {
        let input = "12oz potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 12.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test11() {
        let input = "12oz tequila";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 12.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("tequila".to_string()));
    }
    #[test]
    fn test12() {
        let input = "1/2 potato";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 0.5);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("potato".to_string()));
    }
    #[test]
    fn test13() {
        let input = "1.5 cups flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test14() {
        let input = "1.5 potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test15() {
        let input = "1 clove garlic, minced";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(
            ingredient.ingredient,
            Some("clove garlic, minced".to_string())
        );
    }
    #[test]
    fn test16() {
        let input = "1 cup flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test17() {
        let input = "1 garlic clove, sliced in 1/2";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(
            ingredient.ingredient,
            Some("garlic clove, sliced in 1/2".to_string())
        );
    }
    #[test]
    fn test18() {
        let input = "1 tablespoon (3 teaspoons) Sazon seasoning blend (recommended: Goya) with Mexican and Spanish foods in market";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(
            ingredient.quantities[0].unit,
            Some("tablespoon".to_string())
        );
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("Sazon seasoning blend (recommended: Goya) with Mexican and Spanish foods in market".to_string())
        );
    }
    #[test]
    fn test19() {
        let input = "2 (28 ounce) can crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 56.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("can crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test20() {
        let input = ".25 cups flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 0.25);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test21() {
        let input = "2 cups of potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 2.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test22() {
        let input = "2 eggs, beaten";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 2.);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("eggs, beaten".to_string()));
    }
    #[test]
    fn test23() {
        let input = "3 28 ounce cans of crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 84.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("cans of crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test24() {
        let input = "5 3/4 pinches potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 5.75);
        assert_eq!(ingredient.quantities[0].unit, Some("pinch".to_string()));
        assert_eq!(
            ingredient.quantities[0].unit_type,
            Some(UnitType::Imprecise)
        );
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test25() {
        let input = ".5 potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 0.5);
        assert_eq!(ingredient.quantities[0].unit, None);
        assert_eq!(ingredient.quantities[0].unit_type, None);
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test26() {
        let input = "a cup of flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test27() {
        let input = "ground black pepper to taste";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert!(ingredient.quantities.is_empty());
        assert_eq!(
            ingredient.ingredient,
            Some("ground black pepper to taste".to_string())
        );
    }
    #[test]
    fn test28() {
        let input = "one 28 ounce can crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 28.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("can crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test29() {
        let input = "one cup flour";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("cup".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("flour".to_string()));
    }
    #[test]
    fn test30() {
        let input = "three 28 ounce cans crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 84.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("cans crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test31() {
        let input = "two 28 ounce cans crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 56.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("cans crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test32() {
        let input = "two five ounce can crushed tomatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 10.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("can crushed tomatoes".to_string())
        );
    }
    #[test]
    fn test33() {
        let input = "1kg / 2lb 4oz potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("kilogram".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test34() {
        let input = "2lb 4oz potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 2.);
        assert_eq!(ingredient.quantities[0].unit, Some("pound".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_relative_eq!(ingredient.quantities[1].amount, 4.);
        assert_eq!(ingredient.quantities[1].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[1].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test35() {
        let input = "2lb 4oz (1kg) potatoes";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 2.);
        assert_eq!(ingredient.quantities[0].unit, Some("pound".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_relative_eq!(ingredient.quantities[1].amount, 4.);
        assert_eq!(ingredient.quantities[1].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[1].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("potatoes".to_string()));
    }
    #[test]
    fn test36() {
        let input = "1-1/2 ounce vanilla ice cream";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("vanilla ice cream".to_string()));
    }
    #[test]
    fn test37() {
        let input = "1-½ ounce vanilla ice cream";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("vanilla ice cream".to_string()));
    }
    #[test]
    fn test38() {
        let input = "apple";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert!(ingredient.quantities.is_empty());
        assert_eq!(ingredient.ingredient, Some("apple".to_string()));
    }
    #[test]
    fn test39() {
        let input = "1-½ ounce vanilla ice cream";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.5);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(ingredient.ingredient, Some("vanilla ice cream".to_string()));
    }
    #[test]
    fn test40() {
        let input = "3-⅝ ounces, weight feta cheese, crumbled/diced";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 3.625);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("weight feta cheese, crumbled/diced".to_string())
        );
    }
    #[test]
    fn test41() {
        let input = "3-⅝ ounces, weight feta cheese, crumbled/diced";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 3.625);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("weight feta cheese, crumbled/diced".to_string())
        );
    }
    #[test]
    fn test42() {
        let input = "16-ounce can of sliced pineapple";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 16.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("can of sliced pineapple".to_string())
        );
    }
    #[test]
    fn test43() {
        let input = "750ml/1 pint 7fl oz hot vegetable stock";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 750.);
        assert_eq!(
            ingredient.quantities[0].unit,
            Some("milliliter".to_string())
        );
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert_eq!(
            ingredient.ingredient,
            Some("hot vegetable stock".to_string())
        );
    }
    #[test]
    fn test44() {
        let input = "pinch salt";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("pinch".to_string()));
        assert_eq!(
            ingredient.quantities[0].unit_type,
            Some(UnitType::Imprecise)
        );
        assert_eq!(ingredient.ingredient, Some("salt".to_string()));
    }
    #[test]
    fn test45() {
        let input = "4 (16 ounce) t-bone steaks, at room temperature";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 64.);
        assert_eq!(ingredient.quantities[0].unit, Some("ounce".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert_eq!(
            ingredient.ingredient,
            Some("t-bone steaks, at room temperature".to_string())
        );
    }
    #[test]
    fn test46() {
        let input = "5g";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 5.);
        assert_eq!(ingredient.quantities[0].unit, Some("gram".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert!(ingredient.ingredient.is_none());
    }
    #[test]
    fn test47() {
        let input = "30 cal";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 30.);
        assert_eq!(ingredient.quantities[0].unit, Some("calorie".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert!(ingredient.ingredient.is_none());
    }
    #[test]
    fn test48() {
        let input = "2.5 kcal";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 2.5);
        assert_eq!(ingredient.quantities[0].unit, Some("calorie".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert!(ingredient.ingredient.is_none());
    }
    #[test]
    fn test49() {
        let input = "50 joules";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 50.);
        assert_eq!(ingredient.quantities[0].unit, Some("joule".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert!(ingredient.ingredient.is_none());
    }
    #[test]
    fn test50() {
        let input = "1 kJ";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 1.);
        assert_eq!(ingredient.quantities[0].unit, Some("kilojoule".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::Metric));
        assert!(ingredient.ingredient.is_none());
    }
    #[test]
    fn test51() {
        let input = "20 gallons";
        let ingredient = Ingredient::parse(input);
        assert!(ingredient.is_ok());
        let ingredient = ingredient.unwrap();
        assert_relative_eq!(ingredient.quantities[0].amount, 20.);
        assert_eq!(ingredient.quantities[0].unit, Some("gallon".to_string()));
        assert_eq!(ingredient.quantities[0].unit_type, Some(UnitType::English));
        assert!(ingredient.ingredient.is_none());
    }
}
