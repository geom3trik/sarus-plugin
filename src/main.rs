use sarus::{frontend::Declaration, jit, parser, run_string};

use tuix::*;

mod ui;
use ui::*;

const STYLE: &str = r#"
    .node {
        background-color: #303030;
    }
    .socket {
        background-color: green;
    }
    
    .node_label {
        background-color: #303099;
    }
    popup {
        background-color: #d2d2d2;
    }
    list>button {
        height: 30px;
        child-space: 1s;
        color: black;
        background-color: #d2d2d2;
    }
    list>button:hover {
        background-color: #e2e2e2;
    }
    list>button:active {
        background-color: #c2c2c2;
    }
"#;


fn main() -> anyhow::Result<()> {
    // Create the JIT instance, which manages all generated functions and data.
    let mut jit = jit::JIT::default();

    let code = r#"
fn main(a) -> (b) {
    b = 0.05 * a
}

fn note_A() -> (b) {
    b = 440.0
} 

fn note_Cs() -> (b) {
    b = 554.37
} 

fn note_E() -> (b) {
    b = 659.25
} 

fn double(a) -> (b) {
    b = 2.0 * a
}

fn divide_three(a) -> (b) {
    b = a / 3.0
}

fn tanh_node(a) -> (b) {
    b = tanh(a)
}

fn sin_node(a) -> (b) {
    b = sin(a)
}

fn const_01() -> (b) {
    b = 0.0001
}

fn mul_node(a, b) -> (c) {
    c = a * b
}

fn add_node(a, b) -> (c) {
    c = a + b
}

fn sine_wave(n, f) -> (a) {
    a = sin(2.0 * 3.1415926 * f * (n / 48000.0))
}

"#;

    // Run string with jit instance.
    // This function is unsafe since it relies on the caller to provide it with the correct
    // input and output types. Using incorrect types at this point may corrupt the program's state.
    // Check out run_string() source if you need to separate out execution and parsing steps
    let result: f64 = unsafe { run_string(&mut jit, code, "main", (100.0f64, 200.0f64))? };

    println!("the answer is: {}", result);

    let window_description = WindowDescription::new().with_title("Audio Nodes");
    let app = Application::new(window_description, move |state, window| {
        
        state.add_theme(STYLE);
        window.set_background_color(state, Color::rgb(30,30,30));
        let node_app = NodeApp::new(code).build(state, window, |builder| builder);

        let ast: Vec<Declaration> = parser::program(code).expect("Failed to parse code");

        for decl in ast.into_iter() {
            node_app.emit(state, AppEvent::AddNode(NodeDesc {
                name: decl.name.to_string(),
                inputs: decl.params.clone(),
                outputs: decl.returns.clone(),
            }));
        }
        

        //println!("the answer is: {}", run_file(state, node_app, &mut jit).expect("Failed"));
    });
    app.run();


    Ok(())
}
