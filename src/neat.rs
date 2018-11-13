use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use rand::distributions::Normal;
use rand::thread_rng;
use rand::Rng;

use serde_derive::{Serialize, Deserialize};

const FACTOR_DISJOINT: f64 = 1.;
const FACTOR_WDIFF: f64 = 0.2;
const DIFF_THRESH: f64 = 4.;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub nr_ins: usize,
    pub nr_outs: usize,
    pub connections: HashMap<usize, Connection>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
    pub disabled: bool,
}

impl Genome {
    pub fn init(n_inputs: usize, n_outputs: usize) -> (Genome, usize) {
        let mut rng = thread_rng();
        let weight_dist = Normal::new(0., 1.);

        let mut connections = HashMap::new();
        let mut id = 0;
        for i in 0..n_outputs {
            for j in 0..(n_inputs + 1) {
                connections.insert(
                    id,
                    Connection {
                        from: j,
                        to: i + n_inputs + 1,
                        weight: rng.sample(weight_dist),
                        disabled: false,
                    },
                );
                id += 1;
            }
        }

        (
            Genome {
                connections,
                nr_ins: n_inputs,
                nr_outs: n_outputs,
            },
            id,
        )
    }

    pub fn evaluate(&self, inputs: &[f64]) -> Vec<f64> {
        assert_eq!(self.nr_ins, inputs.len());

        let mut res: Vec<f64> = Vec::new();

        for i in 0..self.nr_outs {
            res.push(self.evaluate_node(self.nr_ins + i + 1, inputs));
        }

        res
    }

    pub fn evaluate_node(&self, node: usize, inputs: &[f64]) -> f64 {
        if node < self.nr_ins {
            return inputs[node];
        }
        if node == self.nr_ins {
            return 1.; // Bias node
        }

        let mut sum = 0.;
        for connection in self.connections.values() {
            if connection.disabled {
                continue;
            }
            if connection.to == node {
                sum += self.evaluate_node(connection.from, inputs) * connection.weight;
            }
        }
        (5. * sum).tanh()
    }

    pub fn merge_with(&self, other: &Genome, other_better: Option<bool>) -> Genome {
        let mut rng = thread_rng();
        let mut new_connections = HashMap::new();

        let my_id_max = *self.connections.keys().max().unwrap_or(&0);
        let other_id_max = *other.connections.keys().max().unwrap_or(&0);

        for i in 0..my_id_max.max(other_id_max) + 1 {
            let other_better_ = other_better.unwrap_or(rng.gen());

            match (
                self.connections.contains_key(&i),
                other.connections.contains_key(&i),
            ) {
                (true, true) => {
                    let connection = if rng.gen() {
                        self.connections[&i]
                    } else {
                        other.connections[&i]
                    };

                    new_connections.insert(i, connection);
                }
                (true, false) if !other_better_ => {
                    new_connections.insert(i, self.connections[&i]);
                }
                (false, true) if other_better_ => {
                    new_connections.insert(i, other.connections[&i]);
                }
                _ => {}
            }
        }

        Genome {
            nr_ins: self.nr_ins,
            nr_outs: self.nr_outs,
            connections: new_connections,
        }
    }

    pub fn mutate_add_node(&mut self, g_id: &mut usize) {
        let mut rng = thread_rng();

        let connection = &self.connections.keys().collect::<Vec<_>>()
            [rng.gen_range(0, self.connections.keys().len())]
        .clone();

        self.connections.get_mut(connection).unwrap().disabled = true;

        let new_node_idx = self
            .connections
            .values()
            .flat_map(|x| vec![x.to, x.from])
            .max()
            .unwrap_or(0)
            + 1;

        *g_id += 1;
        self.connections.insert(
            *g_id,
            Connection {
                from: self.connections[connection].from,
                to: new_node_idx,
                weight: 1.,
                disabled: false,
            },
        );

        *g_id += 1;
        self.connections.insert(
            *g_id,
            Connection {
                from: new_node_idx,
                to: self.connections[connection].to,
                weight: self.connections[connection].weight,
                disabled: false,
            },
        );
    }

    pub fn mutate_add_connection(&mut self, g_id: &mut usize) {
        let mut rng = thread_rng();

        'outer: for i in 0..40 {
            let c_ids = self.connections.values().collect::<Vec<_>>();

            let from = c_ids[rng.gen_range(0, c_ids.len())].from;
            let to = c_ids[rng.gen_range(0, c_ids.len())].to;

            if from == to {
                continue;
            }
            // Check for existing connection
            for conn in self.connections.values() {
                if conn.from == from && conn.to == to {
                    continue 'outer;
                }
            }

            // Check if connecting node_1 to node_2 would create a cycle, ie check if from depends
            // on to.
            let mut stack = vec![from];
            while !stack.is_empty() {
                let curr = stack.remove(stack.len() - 1);
                if curr == to {
                    // Found loop
                    continue 'outer;
                }
                for conn in self.connections.values() {
                    if conn.to == curr {
                        stack.push(conn.from);
                    }
                }
            }

            *g_id += 1;
            self.connections.insert(
                *g_id,
                Connection {
                    from: from,
                    to: to,
                    weight: 0.,
                    disabled: false,
                },
            );
            break;
        }
    }

    pub fn mutate(&mut self, g_id: &mut usize, is_small: bool) {
        let dev = if is_small { 0.02 } else { 0.1 };
        let weight_change_dist = Normal::new(0., dev);

        let mut rng = thread_rng();

        let ch_con = if is_small { 0.1 } else { 0.3 };
        if rng.gen::<f64>() < ch_con {
            self.mutate_add_connection(g_id);
        }

        let ch_node = if is_small { 0.003 } else { 0.05 };
        if rng.gen::<f64>() < ch_node {
            self.mutate_add_node(g_id);
        }

        for connection in self.connections.values_mut() {
            connection.weight += rng.sample(&weight_change_dist);
        }
    }

    pub fn gen_graphviz(&self, name: String) {
        let mut res = vec!["rankdir=\"BT\"".into()];
        for (id, conn) in &self.connections {
            let mut other = "";
            if !conn.disabled {
                other = ",style=bold";
            }
            res.push(format!(
                "{} -> {} [label=\"{:?}: {:.1}\"{}];",
                conn.from, conn.to, id, conn.weight, other
            ));
        }

        let sres = format!("digraph {{ {} }}", res.into_iter().collect::<String>());

        let path = format!("/tmp/{}.gv", name);
        let mut f = File::create(&path).unwrap();
        write!(f, "{}", sres);

        Command::new("dot")
            .arg("-Tsvg")
            .arg(path)
            .arg("-o")
            .arg(format!("/tmp/graphs/{}.svg", name))
            .output();
    }

    fn dist(&self, other: &Genome) -> f64 {
        // We treat disjoint and excess as the same thing
        let mut nr_disjoint = 0;
        let mut weight_diff: f64 = 0.;

        for (id, conn) in &self.connections {
            if !other.connections.contains_key(id) {
                nr_disjoint += 1;
            } else {
                weight_diff += (conn.weight - other.connections[id].weight).abs();
            }
        }
        for (id, conn) in &other.connections {
            if !self.connections.contains_key(id) {
                nr_disjoint += 1;
            } else {
                weight_diff += (conn.weight - self.connections[id].weight).abs();
            }
        }

        nr_disjoint as f64 * FACTOR_DISJOINT + weight_diff * FACTOR_WDIFF
    }
}

pub fn class_species(
    population: Vec<Genome>,
    old_species: Vec<Vec<(Genome, usize)>>,
) -> Vec<Vec<(Genome, usize)>> {
    let mut species: Vec<Vec<(Genome, usize)>> = vec![Vec::new(); old_species.len()];

    for (i, genome) in population.into_iter().enumerate() {
        let mut spec_idx = None;

        'spec: for n in 0..species.len() {
            let cmp;
            if n < old_species.len() {
                cmp = &old_species[n][0].0;
            } else {
                cmp = &species[n][0].0;
            }
            if cmp.dist(&genome) < DIFF_THRESH {
                spec_idx = Some(n);
            }
        }

        if let Some(idx) = spec_idx {
            species[idx].push((genome, i));
        } else {
            species.push(vec![(genome, i)]);
        }
    }

    species.retain(|x| !x.is_empty());

    species
}

pub fn next_generation(
    mut species: Vec<Vec<(Genome, usize)>>,
    fitnesses: Vec<f64>,
    g_id: &mut usize,
    verbose: bool,
) -> Vec<Genome> {
    let mut rng = thread_rng();

    let species_orig_size = species.iter().map(Vec::len).collect::<Vec<_>>();

    // Remove bottom 50% of each species
    for x in &mut species {
        x.sort_unstable_by(|(_, idx1), (_, idx2)| {
            fitnesses[*idx2].partial_cmp(&fitnesses[*idx1]).unwrap()
        });
        x.truncate(x.len() / 2 + 1);
    }
    species.retain(|x| !x.is_empty());

    let average_species_size =
        species.iter().map(Vec::len).sum::<usize>() as f64 / species.len() as f64;

    let mut adj_fitness = vec![0.; fitnesses.len()];

    for x in &species {
        let len = x.len();
        for (ind, idx) in x {
            adj_fitness[*idx] = fitnesses[*idx] / len as f64;
        }
    }

    let mut species_fitness = Vec::new();
    for x in &species {
        let mut tot_fit = 0.;
        for (_, idx) in x {
            tot_fit += adj_fitness[*idx];
        }
        species_fitness.push(tot_fit);
    }

    let average_species_fitness =
        species_fitness.iter().sum::<f64>() / species_fitness.len() as f64;

    let deviation = species_fitness
        .iter()
        .map(|x| (x - average_species_fitness).powi(2))
        .sum::<f64>()
        .sqrt();

    let mut offspring: Vec<f64> = Vec::new();
    for (i, sp) in species.iter().enumerate() {
        let val = species_fitness[i];

        let mult = match species_fitness[i] {
            x if x < average_species_fitness - deviation => 0.1,
            x if x < average_species_fitness => 0.5,
            x if x < average_species_fitness + deviation => 1.5,
            _ => 3.,
        };

        offspring.push(val * mult);
    }

    let mut result: Vec<Genome> = Vec::new();

    for (i, sp) in species.iter().enumerate() {
        let mut num_offspring =
            offspring[i] / offspring.iter().sum::<f64>() * fitnesses.len() as f64;
        if rng.gen::<f64>() < num_offspring % 1. {
            num_offspring += 1.;
        }
        let num_offspring = num_offspring as usize;

        let is_small = (sp.len() as f64) < average_species_size;

        if verbose {
            println!(
                "Species {}: size: {} count {} ({}) fitness {}, offspring: {}",
                i,
                if is_small { "small" } else { "large" },
                sp.len(),
                species_orig_size[i],
                species_fitness[i],
                num_offspring
            );
        }

        for i in 0..num_offspring {
            let (ind_1, idx_1) = &sp[rng.gen_range(0, sp.len())];
            let (ind_2, idx_2) = &sp[rng.gen_range(0, sp.len())];

            if rng.gen::<f64>() < 0.4 {
                result.push(ind_1.clone());
                continue;
            }

            let other_better = match fitnesses[*idx_1].partial_cmp(&fitnesses[*idx_2]) {
                Some(Ordering::Less) => Some(true),
                Some(Ordering::Equal) => None,
                Some(Ordering::Greater) => Some(false),
                None => None,
            };

            let mut merged = ind_1.merge_with(&ind_2, other_better);
            if rng.gen::<f64>() < 0.4 {
                merged.mutate(g_id, is_small);
            }

            result.push(merged);
        }
    }

    if verbose {
        println!("Fitness deviation: {:?}", deviation);
    }

    rng.shuffle(&mut result);

    result
}

const TABLE: &[(f64, f64, f64)] = &[(0., 0., 0.), (0., 1., 1.), (1., 0., 1.), (1., 1., 0.)];

#[allow(unused)]
pub fn test_xor() {
    let pop = vec![Genome::init(2, 1); 1000];
    let mut g_id = pop[0].1;
    let mut pop = pop.into_iter().map(|x| x.0).collect::<Vec<_>>();

    let mut last_best = pop[0].clone();

    let mut old_species = class_species(pop.clone(), vec![]);

    for extinction in 0..1 {
        for i in 0..100 {
            println!("== ITER {} ==", i);
            let mut fitness: Vec<f64> = Vec::new();
            let mut best: Option<(usize, f64)> = None;

            for (i, ind) in pop.iter().enumerate() {
                let mut fit = 0.;
                for &(i1, i2, out) in TABLE {
                    fit += (ind.evaluate(&[i1, i2])[0] - out).powi(2);
                }

                fit = 1. / (fit + 1.);
                // fit = fit.powi(5);
                fitness.push(fit);

                match best {
                    Some((_, x)) if x < fit => {
                        best = Some((i, fit));
                    }
                    None => {
                        best = Some((i, fit));
                    }
                    _ => {}
                }
            }

            last_best = pop[best.unwrap().0].clone();

            // if i % 10 == 0 {
            //     let species = class_species(pop.clone());
            //     for (j, sp) in species.iter().enumerate() {
            //         if sp.len() < 10 {
            //             continue;
            //         }

            //         let mut best = sp[0].1;
            //         for (_, idx) in sp {
            //             if fitness[*idx] > fitness[best] {
            //                 best = *idx;
            //             }
            //         }

            //         pop[best].gen_graphviz(format!("{}-sp{}", i, j));
            //     }
            // }

            println!(
                "Average fitness: {:?}",
                fitness.iter().sum::<f64>() / fitness.len() as f64
            );
            if let Some(best) = best {
                println!("Best fitness: {:?}", best.1);
            }

            let species = class_species(pop, old_species);
            old_species = species.clone();

            pop = next_generation(species, fitness, &mut g_id, i % 5 == 0);

            println!("Done");
        }

        // pop = vec![last_best.clone().clone(); pop.len()];
        // println!("");
        // println!("");
        // println!(" ===== BOOM ===== ");
        // println!("");
        // println!("");
    }
    for &(i1, i2, _out) in TABLE {
        println!("{}, {} => {}", i1, i2, last_best.evaluate(&[i1, i2])[0]);
    }
    last_best.gen_graphviz("best".into());
}
