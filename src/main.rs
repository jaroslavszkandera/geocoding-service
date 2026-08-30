use geocoding::{Forward, Openstreetmap, Point, Reverse};

fn main() {
    let osm = Openstreetmap::new();
    let address = "Třinec";
    let forward_res: Result<Vec<Point<f64>>, geocoding::GeocodingError> = osm.forward(&address);
    dbg!(&forward_res);
    let reverse_res = osm.reverse(forward_res.unwrap().first().expect("None"));
    dbg!(&reverse_res);
    // assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
    // println!("Hello, world!");
}
