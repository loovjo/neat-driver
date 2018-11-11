use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use rand::distributions::Normal;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug)]
pub struct Genome {
    pub nr_ins: usize,
    pub nr_outs: usize,
    pub connections: HashMap<usize, Connection>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
    pub disabled: bool,
}

impl Genome {
    pub fn init(n_inputs: usize, n_outputs: usize) -> Genome {
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

        Genome {
            connections,
            nr_ins: n_inputs,
            nr_outs: n_outputs,
        }
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
        sum.tanh()
    }

    pub fn merge_with(&self, other: &Genome, other_better: Option<bool>) -> Genome {
        let mut rng = thread_rng();
        let mut new_connections = HashMap::new();

        let my_id_max = *self.connections.keys().max().unwrap_or(&0);
        let other_id_max = *other.connections.keys().max().unwrap_or(&0);

        for i in 0..my_id_max.max(other_id_max) {
            let other_better_ = other_better.unwrap_or(rng.gen());

            match (
                self.connections.contains_key(&i),
                other.connections.contains_key(&i),
            ) {
                (true, true) => {
                    assert_eq!(self.connections[&i].from, other.connections[&i].from);
                    assert_eq!(self.connections[&i].to, other.connections[&i].to);

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

    pub fn gen_graphviz(&self, name: String) {
        let mut res = vec!["rankdir=\"BT\"".into()];
        for (id, conn) in &self.connections {
            res.push(format!("{} -> {} [label={:?}];", conn.from, conn.to, id));
        }

        let sres = format!("digraph {{ {} }}", res.into_iter().collect::<String>());

        let path = format!("/tmp/{}.gv", name);
        let mut f = File::create(&path).unwrap();
        write!(f, "{}", sres);

        Command::new("dot")
            .arg("-Tsvg")
            .arg(path)
            .arg("-o")
            .arg(format!("graphs/{}.svg", name))
            .output();

    }
}

pub fn test() {
    //    3
    //   /|\
    //  / 4 \
    // | /|  |
    // 0/ 1  2
    //
    // Crossing with
    //
    //
    //    3
    //   /|\
    //  / 5 \
    // / /|  \
    // |/ 4  |
    // |  |\ |
    // 0  1 \2

    let mut c1 = HashMap::new();
    c1.insert(
        0,
        Connection {
            from: 0,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c1.insert(
        1,
        Connection {
            from: 1,
            to: 3,
            weight: 1.,
            disabled: true,
        },
    );
    c1.insert(
        2,
        Connection {
            from: 2,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c1.insert(
        3,
        Connection {
            from: 1,
            to: 4,
            weight: 1.,
            disabled: false,
        },
    );
    c1.insert(
        4,
        Connection {
            from: 4,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c1.insert(
        7,
        Connection {
            from: 0,
            to: 4,
            weight: 1.,
            disabled: false,
        },
    );

    let g1 = Genome {
        connections: c1,
        nr_ins: 2,
        nr_outs: 1,
    };

    let mut c2 = HashMap::new();
    c2.insert(
        0,
        Connection {
            from: 0,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        1,
        Connection {
            from: 1,
            to: 3,
            weight: 1.,
            disabled: true,
        },
    );
    c2.insert(
        2,
        Connection {
            from: 2,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        3,
        Connection {
            from: 1,
            to: 4,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        4,
        Connection {
            from: 4,
            to: 3,
            weight: 1.,
            disabled: true,
        },
    );
    c2.insert(
        5,
        Connection {
            from: 4,
            to: 5,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        6,
        Connection {
            from: 5,
            to: 3,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        8,
        Connection {
            from: 2,
            to: 4,
            weight: 1.,
            disabled: false,
        },
    );
    c2.insert(
        9,
        Connection {
            from: 0,
            to: 5,
            weight: 1.,
            disabled: false,
        },
    );

    let g2 = Genome {
        connections: c2,
        nr_ins: 2,
        nr_outs: 1,
    };

    g1.gen_graphviz("g1".into());
    g2.gen_graphviz("g2".into());

    let merged = g1.merge_with(&g2, Some(true));

    merged.gen_graphviz("merged".into());
}
