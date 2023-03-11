use eqsolver::single_variable::FDNewton;
use dms_coordinates::{DMS, Bearing::*};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufReader, BufRead, Error};
mod calc_dimensions;
use calc_dimensions::calc_horizontal_dimensions::*;

#[derive(Debug)]
pub enum SightType {
    Stopping,
    Passing,
    Decision,
}

#[derive(Debug)]
pub enum Angle {
    DecimalDegrees (f64),
    Dms (DMS),
    Radians (f64),
}

impl Angle {
    pub fn create_dms(line: &str) -> Angle {
        let delimiters = "'\"d";
        let parts: Vec<&str> = line.split(|c| delimiters.contains(c)).collect();
        
        Angle::Dms(DMS { 
            degrees: parts[0].parse::<i32>().unwrap(), 
            minutes: parts[1].parse::<i32>().unwrap(), 
            seconds: parts[2].parse::<f64>().unwrap(), 
            bearing: East 
        })
    }

    pub fn value(&self) -> f64 {
        match self {
            Angle::DecimalDegrees(n) => *n,
            Angle::Radians(n) => *n,
            Angle::Dms(_n) => panic!("Convert dms to radians or decimal degrees first."),
        }
    }

    pub fn to_dms(self) -> Self {
        match self {
            Angle::DecimalDegrees(ddeg) => Angle::Dms(DMS::from_decimal_degrees(ddeg, false)),
            Angle::Radians(rad) => Angle::Dms(DMS::from_decimal_degrees(rad*180.0/PI, false)),
            _ => self,
        }
    }

    pub fn to_decimal_degrees(self) -> Self {
        match self {
            Angle::Dms(dms_value) => Angle::DecimalDegrees(DMS::to_decimal_degrees(&dms_value)),
            Angle::Radians(rad) => Angle::DecimalDegrees(rad*180.0/PI),
            _ => self,
        }
    }

    pub fn to_radians(self) -> Self {
        match self {
            Angle::Dms(dms_value) => Angle::Radians(DMS::to_decimal_degrees(&dms_value)*PI/180.0),
            Angle::DecimalDegrees(dd_value) => Angle::Radians(dd_value*PI/180.0),
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct HorizontalCurve {
    dimensions: HorizontalDimensions,
    stations: HorizontalStations,
}

#[derive(Debug)]
pub struct HorizontalStations {
    pc: f64, 
    pi: f64,
    pt: f64,

}

#[derive(Debug)]
pub struct HorizontalDimensions {
    radius: f64,
    curve_length: f64,
    tangent_distance: f64,
    long_chord: f64,
    middle_ordinate: f64,
    external: f64,
    curve_length_100: Angle, //(Da)
    curve_angle: Angle, //radians (I)
    sight_distance: Option<f64>,
}

pub fn single_var() {
    let y = 32.0;
    //let f = |x: f64| x.exp() - 1./x; // e^x = 1/x
    let f = |x: f64| x - y*2.0; // e^x = 1/x
    let solution = FDNewton::new(f).solve(0.);
    println!("Solution: {:?}", solution);
}

pub fn parse_input() -> Result<HashMap<String, String>, Error> {
    let input = File::open("input.md")?;
    let buffered = BufReader::new(input);
    let mut arguments = HashMap::new();

    for line in buffered.lines().flatten() {
        let i = line.split_once('=');
        match i {
            None => continue,
            Some(args) => arguments.insert(args.0.to_owned(), args.1.to_owned()),
        };
    }

    //arguments.insert("radius", Some(3.0));
    Ok(arguments)
}

impl HorizontalCurve {
    pub fn create(given: Result<HashMap<String, String>, Error>) -> Result<HorizontalCurve, Error> { //
        let mut given = given?;
        if !given.contains_key("Da") {
            given.insert("Da".to_string(), radius_to_da(given.get("R").expect("missing R (radius) or Da")));
        }
        let da = given.get("Da").unwrap();
        let i = given.get("I").unwrap();
        let pi = given.get("PI").unwrap();

        let dimensions = HorizontalDimensions { 
            radius: calc_radius(da), 
            curve_length: calc_curve_length(da,i), 
            tangent_distance: calc_tangent_distance(da,i), 
            long_chord: calc_long_chord(da,i), 
            middle_ordinate: calc_middle_ordinate(da,i), 
            external: calc_external(da,i), 
            curve_length_100: calc_curve_length_100(da), 
            curve_angle: calc_curve_angle(i),
            sight_distance: None,
        };
        let stations = HorizontalStations { 
            pc: calc_pc(pi, dimensions.tangent_distance), 
            pi: calc_pi(pi), 
            pt: calc_pt(pi, dimensions.tangent_distance, dimensions.curve_length)
        };

        Ok(HorizontalCurve {dimensions, stations})
    }
}

//The stopping sight distances in Table 201.1 should be increased by 20 percent on sustained downgrades steeper than 3 percent and longer than one mile. use figure 201.6
pub fn calc_min_sight_distance(design_speed: i32, sight_type: SightType, sustained_downgrade: bool) -> Result<f64, Error> {
    let mut minimum_sight_distance = match sight_type {
        SightType::Stopping => 1.0,
        SightType::Passing => 2.0,
        SightType::Decision => 3.0,
    };
    if sustained_downgrade {
        minimum_sight_distance *= 1.2;
    }
    Ok(minimum_sight_distance)
}