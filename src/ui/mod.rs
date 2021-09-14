
pub mod node_view;
use std::{collections::HashMap, ops::Index};

pub use node_view::*;

pub mod node_widget;
pub use node_widget::*;

pub mod socket_widget;
pub use socket_widget::*;

use tuix::*;

use sarus::{graph::{Graph, Node, Connection}, run_fn};

const STEP_SIZE: usize = 16usize;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeEvent {
    TrySnap(Entity, Entity),
    ConnectSockets(Entity),
    ConnectInput,
    ConnectOutput,
    //Disconnect(Entity),
    Snap(Entity, Entity),
    Connecting,
    Disconnect,

    AddConnection(ConnectionDesc),
    RemoveConnection(ConnectionDesc),
}

#[derive(PartialEq, Clone)]
pub struct NodeDesc {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(PartialEq)]
pub enum AppEvent {
    AddNode(NodeDesc),
    InsertNode(String),
    Run,
}

#[derive(Debug)]
pub struct NodeDesc2 {
    entity: Entity,
    name: String,
    inputs: Vec<Entity>,
    outputs: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionDesc {
    source: Entity,
    dest: Entity,
    output_socket: Entity,
    input_socket: Entity,
}

// Where everything lives
// TODO - Rename me
pub struct NodeApp {
    graph: Graph,
    node_view: Entity,
    menu: Entity,
    node_descriptions: HashMap<String, NodeDesc>,
    code: String,

    nodes: Vec<NodeDesc2>,
    connections: Vec<ConnectionDesc>,
}

impl NodeApp {
    pub fn new(code: &str) -> Self {
        
        let nodes = vec![
            Node {
                func_name: "INPUT".to_string(),
                id: "INPUT".to_string(),
                port_defaults: vec![0.0],
                position: (0.0, 0.0),
            },
            Node {
                func_name: "OUTPUT".to_string(),
                id: "OUTPUT".to_string(),
                port_defaults: vec![0.0],
                position: (0.0, 0.0),
            },
        ];

        let connections = vec![
            Connection {
                src_node: 0,
                dst_node: 1,
                src_port: 0,
                dst_port: 0,
            },
        ];

        println!("Nodes: {:?}", nodes);
        println!("Connections: {:?}", connections);

        Self {
            graph: Graph::new(code.to_string(), nodes, connections, STEP_SIZE).unwrap(),
            node_view: Entity::null(),
            menu: Entity::null(),
            node_descriptions: HashMap::new(),
            code: code.to_string(),
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        println!("Compile: {} {}", self.nodes.len(), self.connections.len());
        let nodes = self.nodes.iter().map(|node_desc| {
            let mut defaults = node_desc.inputs.iter().map(|_| 0.0f64).collect::<Vec<_>>();
            defaults.append(&mut node_desc.outputs.iter().map(|_|0.0f64).collect::<Vec<_>>());
            let id = if node_desc.name == "INPUT" {
                "INPUT".to_string()
            } else if node_desc.name == "OUTPUT" {
                "OUTPUT".to_string()
            } else {
                node_desc.entity.to_string()
            };

            Node {
                func_name: node_desc.name.clone(),
                id,
                port_defaults: defaults,
                position: (0.0, 0.0),
            }
        }).collect::<Vec<_>>();
        println!("{:?} {:?}", self.nodes, self.connections);
        let connections = self.connections.iter().map(|con_desc| {
            
            let src_port = self.nodes.iter().find(|node_desc| node_desc.entity == con_desc.source).unwrap().outputs.iter().position(|&id| id == con_desc.output_socket).unwrap();
            let dst_port = self.nodes.iter().find(|node_desc| node_desc.entity == con_desc.dest).unwrap().inputs.iter().position(|&id| id == con_desc.input_socket).unwrap();
            
            Connection {
                src_node: self.nodes.iter().position(|node_desc| node_desc.entity == con_desc.source).unwrap(),
                dst_node: self.nodes.iter().position(|node_desc| node_desc.entity == con_desc.dest).unwrap(),
                src_port,
                dst_port,
            }
        }).collect::<Vec<_>>();
        
        println!("Nodes: {:?}", nodes);
        println!("Connections: {:?}", connections);

        self.graph = Graph::new(self.code.clone(), nodes, connections, STEP_SIZE).expect("Failed to rebuild graph");

        self.run();
        
    }

    pub fn run(&mut self) -> anyhow::Result<()> {

        for d in &self.graph.ast {
            println!("{}", d);
        }

        const STEPS: usize = 48000 / STEP_SIZE;
        let mut output_arr = [[0.0f64; STEP_SIZE]; STEPS];
        let mut n = 0;
        for i in 0..STEPS {
            let mut audio_buffer = [0.0f64; STEP_SIZE];
            for j in 0..STEP_SIZE {
                audio_buffer[j] = ((n as f64).powi(2) * 0.000001).sin(); //sound source is sine sweep
                n += 1;
            }
            unsafe { run_fn(&mut self.graph.jit, "graph", &mut audio_buffer)?};
    
            //Collect output audio
            output_arr[i] = audio_buffer;
        }

        //Flatten output audio chunks for saving as wav
        let flat = output_arr
        .iter()
        .flatten()
        .map(|x| *x)
        .collect::<Vec<f64>>();
        self.write_wav(&flat, "graph_test.wav");
        Ok(())
    }

    fn write_wav(&self, samples: &[f64], path: &str) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for sample in samples {
            writer.write_sample(*sample as f32).unwrap();
        }
        writer.finalize().unwrap();
    }
}

impl Widget for NodeApp {
    type Ret = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        let popup = Popup::new()
            .build(state, entity, |builder| {
                builder
                    .set_width(Pixels(100.0))
                    .set_height(Auto)
                    .set_z_order(10)
            });

        self.menu = List::new()
            .build(state, popup, |builder| {
                builder
                    .set_height(Auto)
            });

        self.node_view = NodeView::new().build(state, entity, |builder| {
            builder
        });


        

        let node = NodeWidget::new("INPUT").build(state, self.node_view, |builder| 
            builder
        );

        let mut node_desc2 = NodeDesc2 {
            entity: node,
            name: "INPUT".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        

        let row = Row::new().build(state, node, |builder| 
            builder
                .set_height(Pixels(30.0))
                .set_child_space(Stretch(1.0))
        );

        

        Label::new("src").build(state, row, |builder| 
            builder
                .set_child_space(Stretch(1.0))
                .set_child_right(Pixels(5.0))
                .set_space(Pixels(0.0))
                .set_hoverable(false)
        );

        let output_socket = OutputSocket::new().build(state, row, |builder| 
            builder
                .set_left(Stretch(0.0))
                .set_right(Pixels(-10.0))
        );

        println!("INPUT NODE: {}", output_socket);

        node_desc2.outputs.push(output_socket);

        self.nodes.push(node_desc2);



        let node = NodeWidget::new("OUTPUT").build(state, self.node_view, |builder| 
            builder
        );

        let mut node_desc2 = NodeDesc2 {
            entity: node,
            name: "OUTPUT".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };

        let row = Row::new().build(state, node, |builder| 
            builder
                .set_height(Pixels(30.0))
                .set_child_space(Stretch(1.0))
        );

        let input_socket = InputSocket::new().build(state, row, |builder| 
            builder
                .set_left(Pixels(-10.0))
                .set_right(Stretch(0.0))
        );
    
        Label::new("dst").build(state, row, |builder| 
            builder
                .set_child_space(Stretch(1.0))
                .set_child_left(Pixels(5.0))
                .set_space(Pixels(0.0))
                .set_hoverable(false)
        );

        node_desc2.inputs.push(input_socket);

        self.nodes.push(node_desc2);


        input_socket.emit_to(state, output_socket, NodeEvent::ConnectInput);


        self.node_view
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(app_event) = event.message.downcast() {
            match app_event {

                AppEvent::Run => {
                    self.compile();
                    //self.run().expect("Failed to run");
                }

                AppEvent::AddNode(node) => {

                    let node_name = node.name.clone();

                    self.node_descriptions.insert(node.name.clone(), node.clone());

                    // Add a button to the menu from the node description

                    Button::with_label(&node.name)
                        .on_release(move |_, state, button| {
                            button.emit(state, AppEvent::InsertNode(node_name.clone()));
                            button.emit(state, PopupEvent::Close);
                        })
                        .build(state, self.menu, |builder| 
                            builder
                    );
                }

                AppEvent::InsertNode(name) => {
                    if let Some(node_desc) = self.node_descriptions.get(name) {


                        

                        // Create the node from the description

                        let node = NodeWidget::new(&name).build(state, self.node_view, |builder| 
                            builder
                        );

                        let mut node_desc2 = NodeDesc2 {
                            entity: node,
                            name: name.clone(),
                            inputs: Vec::new(),
                            outputs: Vec::new(),
                        };

                        // let graph_node = Node {
                        //     func_name: node_desc.name.clone(),
                        //     id: node.to_string(),
                        //     port_defaults: node_desc.inputs.iter().map(|_| 0.0).collect(),
                        //     position: (0.0, 0.0),
                        // };

                        // self.graph.nodes.push(graph_node);
            
                        for (index, param) in node_desc.inputs.iter().enumerate() {
            
                            let row = Row::new().build(state, node, |builder| 
                                builder
                                    .set_height(Pixels(30.0))
                                    .set_child_space(Stretch(1.0))
                            );
                        
                            let input_socket = InputSocket::new().build(state, row, |builder| 
                                builder
                                    .set_left(Pixels(-10.0))
                                    .set_right(Stretch(0.0))
                            );
                        
                            Label::new(&param).build(state, row, |builder| 
                                builder
                                    .set_child_space(Stretch(1.0))
                                    .set_child_left(Pixels(5.0))
                                    .set_space(Pixels(0.0))
                                    .set_hoverable(false)
                            );

                            node_desc2.inputs.push(input_socket);
                        }
            
                        for (index, ret) in node_desc.outputs.iter().enumerate() {
                            let row = Row::new().build(state, node, |builder| 
                                    builder
                                        .set_height(Pixels(30.0))
                                        .set_child_space(Stretch(1.0))
                                );
                    
                            Label::new(&ret).build(state, row, |builder| 
                                builder
                                    .set_child_space(Stretch(1.0))
                                    .set_child_right(Pixels(5.0))
                                    .set_space(Pixels(0.0))
                                    .set_hoverable(false)
                            );
                    
                            let output_socket = OutputSocket::new().build(state, row, |builder| 
                                builder
                                    .set_left(Stretch(0.0))
                                    .set_right(Pixels(-10.0))
                            );

                            node_desc2.outputs.push(output_socket);
                        }

                        self.nodes.push(node_desc2);

                        //self.run().expect("Failed to compile and run graph");
                    }
                }

                _=> {}
            }
        }

        if let Some(node_event) = event.message.downcast() {
            match node_event {
                NodeEvent::AddConnection(con_desc) => {
                    //println!("Add {:?}", con_desc);
                    self.connections.push(*con_desc);

                    println!("Add. Connections: {:?}", self.connections);
                }

                NodeEvent::RemoveConnection(con_desc) => {
                    //println!("Remove {:?}", con_desc);
                    //self.connections.remove(&con_desc.input_socket);
                    let pos = self.connections.iter().position(|cd| cd.input_socket == con_desc.input_socket).unwrap();
                    self.connections.remove(pos);
                    println!("Remove. Connections: {:?}", self.connections);
                }

                _=> {}
            }
        }

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseUp(button) if *button == MouseButton::Right => {
                    entity.emit_to(state, self.menu, PopupEvent::OpenAtCursor);
                }

                _=> {}
            }
        }
    }
}