extern crate rand;
extern crate image;
extern crate serde;
extern crate serde_json;

use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;
use std::collections::HashMap;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use scoped_threadpool::Pool;
use std::sync::{Arc, Mutex};

////////////////////////////////////////////////////////////////////////
// constants.json info
#[derive(Deserialize, Copy, Clone)]
struct ConstVals {
    n: usize,
    sz: usize,
    age_dist: [usize;16],
    age_vals: [[u32; 3];8],
    city_sizes: [u32;27],
    start_seed: usize,
    n_steps: u32,
    lockdown_start: [u32;8],
    lockdown_end: [u32;8],
    to_peak: u32,
    cell_threshold: u8,
    jump_threshold: u32,
    noninfective: u32,
    save_images: bool,
    image_size: u32,
}

////////////////////////////////////////////////////////////////////////
/// random walk, band is centred on target, probability of step towards target
/// increases nearer x is to the edge. returns -1, 0 or +1
fn rand_step(rng: &mut ThreadRng, x: i32, target: i32, band: u32) -> i32 {
    if band == 0 || rng.gen_range(0, 3) <= 1 { // 2 in 3 doesn't move
        return 0;
    }
    let half_band = (band as i32) / 2;
    if x > target + half_band {
        return -1;
    }
    if x < target - half_band {
        return 1;
    }
    if rng.gen_ratio((target + half_band - x) as u32, band) {
        return 1;
    }
    -1
}
////////////////////////////////////////////////////////////////////////
struct Point {
  x: i32,
  y: i32,
}
impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point {x, y}
    }
}

////////////////////////////////////////////////////////////////////////
struct Person {
    infect_start: Option<u32>, // None prior to infection ie no immunity
    severity: i32, // signed so subraction simpler 
    max_severity: i32,
    dead: bool,
    posn: Point,
    home: Point,
    age: usize, //in decades
}
impl Person {
    fn new(x: i32, y: i32, age: usize) -> Person {
        Person {
            infect_start: None,
            severity: 0,
            max_severity: 0,
            dead: false,
            posn: Point::new(x, y),
            home: Point::new(x, y),
            age,// from age distribution
        }
    }
}

////////////////////////////////////////////////////////////////////////
/// held in the grid HasMap
struct Cell {
    infection: u8, // 0-255
    next_infection: u8,
}
impl Cell {
    fn new() -> Cell {
        Cell {infection: 0, next_infection: 0}
    }
}

////////////////////////////////////////////////////////////////////////
/// 
struct City {
    centre: Point,
    size: u32,
}
impl City {
    fn new(pop: &u32, rng: &mut ThreadRng, grid_sz: usize, perunit: u32) -> City {
        let x = rng.gen_range(0, grid_sz);
        let y = rng.gen_range(0, grid_sz);
        let size: u32 = ((pop * perunit / 2) as f32).sqrt() as u32;
        City {
            centre: Point::new(x as i32, y as i32),
            size,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
//
///////////////////////////////////////////////////////////////////////////////
fn main() {
    // scoped thread_pool
    let mut pool = Pool::new(4);
    // load constants from json file (in 'current' directory)
    let file = File::open("constants.json").unwrap();
    let reader = BufReader::new(file);
    let C:ConstVals = serde_json::from_reader(reader).unwrap();
    // setup grid, city_list and population
    let mut rng = thread_rng();
    let grid: Arc<Mutex<HashMap<(i32, i32), Cell>>> = Arc::new(Mutex::new(
            HashMap::with_capacity(C.sz * C.sz)));
    let mut city_list = Vec::<City>::with_capacity(C.city_sizes.len());
    let mut pop = Vec::<Person>::with_capacity(C.n);
    // 80% of population in cities
    let perunit: u32 = (C.n as u32) * 8 / 10 / C.city_sizes.iter().sum::<u32>();
    for pop_n in C.city_sizes.iter() {
        let city = City::new(pop_n, &mut rng, C.sz, perunit);
        for i in 0..(pop_n * perunit) as usize {
            let x = city.centre.x + rng.gen_range(0, city.size) as i32 - (city.size / 2) as i32;
            let y = city.centre.y + rng.gen_range(0, city.size) as i32 - (city.size / 2) as i32;
            let p = Person::new(x, y, C.age_dist[i % C.age_dist.len()]);
            pop.push(p);
        }
        city_list.push(city);
    }
    let city_list: Arc<Mutex<Vec<City>>> = Arc::new(Mutex::new(city_list));
    for i in pop.len()..C.n {
        let x = rng.gen_range(0, C.sz) as i32;
        let y = rng.gen_range(0, C.sz) as i32;
        let p = Person::new(x, y, C.age_dist[i % C.age_dist.len()]);
        pop.push(p);
    }
    // seed infections in population
    for _i in 0..C.start_seed {
        let n = rng.gen_range(0, C.n);
        pop[n].infect_start = Some(0);
        pop[n].severity = 6;
    }
    // random walk population
    println!("     day, infected%, recovred%,     dead%");
    let imgbuf = Arc::new(Mutex::new(
        image::ImageBuffer::new(C.image_size, C.image_size)));
    let mut ninfd_vec: Vec<u32> = vec![];
    let mut nrecd_vec: Vec<u32> = vec![];
    let mut ndead_vec: Vec<u32> = vec![];
    let mut lockdown_ix = 0;
    for k in 0..C.n_steps {
        let mob_ix = if k >= C.lockdown_start[lockdown_ix]
                     && k < C.lockdown_end[lockdown_ix] {
            1 } else {
                if k >= C.lockdown_end[lockdown_ix]
                        && (lockdown_ix + 1) < C.lockdown_start.len() {
                    lockdown_ix += 1; // move on to checking next lockdown
                }
            0 };

        if C.save_images { // clear image
            let mut imgbuf = imgbuf.lock().unwrap();
            for y in 0..C.image_size {
                for x in 0..C.image_size {
                    imgbuf.put_pixel(x, y, image::Rgb([0u8, 0u8, 0u8]));
                }
            }
        }
        let ninfd: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));//num infected
        let nrecd: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));//num recovered
        let ndead: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));//num dead

        pool.scoped(|scoped| {
            for p in &mut pop {
                let city_list = Arc::clone(&city_list);
                let grid = Arc::clone(&grid);
                let ninfd = Arc::clone(&ninfd);
                let nrecd = Arc::clone(&nrecd);
                let ndead = Arc::clone(&ndead);
                let imgbuf = Arc::clone(&imgbuf);

                scoped.execute(move || {
                    //let C = &C.clone(); //normal clone?
                    let mut rng = thread_rng();
                    // TODO progress infection, kill off or recover
                    if p.infect_start == None || p.severity > 0 { //not caught it yet or still ill
                        match p.infect_start {
                            Some(t) => { // i.e. severity must be > 0
                                if C.save_images {
                                    let x = (p.posn.x * C.image_size as i32 / C.sz as i32) as u32 % C.image_size;
                                    let y = (p.posn.y * C.image_size as i32 / C.sz as i32) as u32 % C.image_size;
                                    let mut imgbuf = imgbuf.lock().unwrap();
                                    imgbuf.put_pixel(x, y, image::Rgb([0u8, 128u8, 0u8]));//green=infection
                                }
                                let target: i32 = if k < (t + C.to_peak) {13} else {0};
                                p.severity += rand_step(&mut rng, p.severity, target, 30);
                                if p.severity > p.max_severity {
                                    p.max_severity = p.severity;
                                }
                                if p.severity >= C.age_vals[p.age][2] as i32 { //fatal!
                                    p.severity = 0;
                                    p.dead = true;
                                    let mut ndead = ndead.lock().unwrap();
                                    *ndead += 1;
                                } else {
                                    let mut ninfd = ninfd.lock().unwrap();
                                    *ninfd += 1;
                                }
                            },
                            _ => {} 
                        }
                        // then move around
                        let mobility = C.age_vals[p.age][mob_ix];
                        if mobility >= C.jump_threshold && rng.gen_ratio(mobility, 10) {
                            let dice = rng.gen_range(0, 200);
                            if dice <= mobility { // jump
                                let city_list = city_list.lock().unwrap();
                                let dest = &city_list[rng.gen_range(0, city_list.len())].centre;
                                p.posn.x = dest.x;
                                p.posn.y = dest.y;
                            } else if dice < 50 { // chance of returning home
                                p.posn.x = p.home.x;
                                p.posn.y = p.home.y;
                            }
                        }
                        p.posn.x += rand_step(&mut rng, p.posn.x, p.home.x, mobility);
                        p.posn.y += rand_step(&mut rng, p.posn.y, p.home.y, mobility);
                        let key = (p.posn.x, p.posn.y);
                        let mut grid = grid.lock().unwrap();
                        match p.infect_start {
                            Some(t) => { // already infected - pass it on if after incubation
                                if k > (t + C.noninfective) {
                                    let cell = grid.entry(key).or_insert(Cell::new());
                                    let delta = p.severity as u8;
                                    if (cell.next_infection as u32) + (delta as u32) < 255 {
                                        cell.next_infection += delta;
                                    }
                                }
                            },
                            None => { // not infected, check grid cell
                                match grid.get(&key) {//TODO depends
                                    Some(cell) => {
                                        if cell.infection > C.cell_threshold {
                                            p.infect_start = Some(k);
                                            p.severity = 1; //TODO depends
                                        }
                                    },
                                    None => {},
                                }
                            }
                        }
                    } else { // caught it but severity now 0. either dead or immune!
                        if p.dead {
                            {
                                let mut ndead = ndead.lock().unwrap();
                                *ndead += 1;
                            }
                            if C.save_images { // draw a white dot for a death
                                let x = (p.posn.x * C.image_size as i32 / C.sz as i32) as u32 % C.image_size;
                                let y = (p.posn.y * C.image_size as i32 / C.sz as i32) as u32 % C.image_size;
                                let mut imgbuf = imgbuf.lock().unwrap();
                                imgbuf.put_pixel(x, y, image::Rgb([255u8, 255u8, 255u8]));//red=dead
                            }
                        } else {
                            let mut nrecd = nrecd.lock().unwrap();
                            *nrecd += 1;
                        }
                    }
                });
            }
        });

        let mut grid = grid.lock().unwrap();
        let ninfd = ninfd.lock().unwrap();
        let nrecd = nrecd.lock().unwrap();
        let ndead = ndead.lock().unwrap();
        if C.save_images {
            let mut imgbuf = imgbuf.lock().unwrap();
            if k >= 1 {
                ninfd_vec.push(*ninfd);
                nrecd_vec.push(*nrecd);
                ndead_vec.push(*ndead);
                for q in 0..ninfd_vec.len() {
                    let x: u32 = q as u32 * C.image_size / C.n_steps;
                    let y_ninfd = C.image_size - ninfd_vec[q] * C.image_size / C.n  as u32 - 10;
                    let y_nrecd = C.image_size - nrecd_vec[q] * C.image_size / C.n as u32 - 10;
                    let y_ndead = C.image_size - 100 * ndead_vec[q] * C.image_size / C.n as u32 - 10;
                    for r in 0..4 {
                        for s in 0..4 {
                            imgbuf.put_pixel(x + r, y_ninfd + s, image::Rgb([0u8, 255u8, 0u8]));
                            imgbuf.put_pixel(x + r, y_nrecd + s, image::Rgb([255u8, 255u8, 0u8]));
                            imgbuf.put_pixel(x + r, y_ndead + s, image::Rgb([255u8, 0u8, 0u8]));
                        }
                    }
                }
            }
            imgbuf.save(format!("frames/fr{:03}.jpg", k)).unwrap();
        }

        // attenuate contaminated cells and update with new contaminations
        for (_, cell) in grid.iter_mut() {
            cell.infection -= cell.infection / 2; // virus declines by 1/2 each period
            if cell.infection < 2 {
                cell.infection = 0; // then goes completely
            }
            if cell.next_infection > cell.infection { // move over contamination this round
                cell.infection = cell.next_infection;
            }
            cell.next_infection = 0;
        }
        if k % 10 == 0 { // every ten cycles clear uninfected cells from list
            grid.retain(|_, cell| cell.infection > 0);
        }
        let factor = 100.0 / (C.n as f32); // as % of total population
        println!("{:8}, {:8.2}%, {:8.2}%, {:8.2}%",
            k, (*ninfd as f32) * factor,
            (*nrecd as f32) * factor,
            (*ndead as f32) * factor
        );
    }
    let mut peak_histogram = vec![0; 20];
    for p in &mut pop {
        peak_histogram[p.max_severity as usize] += 1;
    }
    println!("histogram of peak severity through population");
    for i in 0..peak_histogram.len() {
        println!("{} {}", i, peak_histogram[i]);
    }
}
