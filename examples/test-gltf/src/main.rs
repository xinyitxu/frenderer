use frenderer::{input, Camera3D, Transform3D};
use glam::*;
use rand::Rng;

// to run, do:
// cargo run --bin test-gltf


/*
pub struct fpcamera {
    pub pitch: f32,
    player_pos: Vec3,
    player_rot: Rotor3,
}
impl fpcamera {
    fn new() -> Self {
        Self {
            pitch: 0.0,
            player_pos: Vec3::zero(),
            player_rot: Rotor3::identity(),
        }
    }
    fn update(&mut self, input: &frenderer::Input, player: &Player) {
        let MousePos { y: dy, .. } = input.mouse_delta();
        self.pitch += DT as f32 * dy as f32 / 10.0;
        // Make sure pitch isn't directly up or down (that would put
        // `eye` and `at` at the same z, which is Bad)
        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.001, PI / 2.0 - 0.001);
        self.player_pos = player.trf.translation;
        self.player_rot = player.trf.rotation;
    }
    fn update_camera(&self, c: &mut Camera) {
        // The camera's position is offset from the player's position.
        let eye = self.player_pos
        // So, <0, 25, 2> in the player's local frame will need
        // to be rotated into world coordinates. Multiply by the player's rotation:
            + self.player_rot * Vec3::new(0.0, 25.0, 2.0);

        // Next is the trickiest part of the code.
        // We want to rotate the camera around the way the player is
        // facing, then rotate it more to pitch is up or down.

        // We need to turn this rotation into a target vector (at) by
        // picking a point a bit "in front of" the eye point with
        // respect to our rotation.  This means composing two
        // rotations (player and camera) and rotating the unit forward
        // vector around by that composed rotation, then adding that
        // to the camera's position to get the target point.
        // So, we're adding a position and an offset to obtain a new position.
        let at = eye + self.player_rot * Rotor3::from_rotation_yz(self.pitch) * Vec3::unit_z();
        *c = Camera::look_at(eye, at, Vec3::unit_y());
    }
}
 */
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    let source = assets_manager::source::FileSystem::new("content").unwrap();
    #[cfg(target_arch = "wasm32")]
    let source = assets_manager::source::Embedded::from(source::embed!("content"));
    let cache = assets_manager::AssetCache::with_source(source);
    let mut frend = frenderer::with_default_runtime(&window);
    let mut input = input::Input::default();
    let fox = cache
        .load::<assets_manager::asset::Gltf>("Fox")
        .unwrap();
    
    let mut camera = Camera3D {
        translation: Vec3 {
            x: 0.0,
            y: 0.0,
            z: -100.0,
        }
        .into(),
        rotation: Quat::from_rotation_y(0.0).into(),
        // 90 degrees is typical
        fov: std::f32::consts::FRAC_PI_2,
        near: 10.0,
        far: 1000.0,
        aspect: 1024.0 / 768.0,
    };
    frend.meshes.set_camera(&frend.gpu, camera);

    let mut rng = rand::thread_rng();
    const COUNT: usize = 10;
    let fox = fox.read();
    let fox_img = fox.get_image_by_index(0);
    let fox_tex = frend.gpu.create_array_texture(
        &[&fox_img.to_rgba8()],
        frenderer::wgpu::TextureFormat::Rgba8Unorm,
        (fox_img.width(), fox_img.height()),
        Some("fox texture"),
    );
    let prim = fox
        .document
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();
    let reader = prim.reader(|b| Some(fox.get_buffer_by_index(b.index())));
    let verts: Vec<_> = reader
        .read_positions()
        .unwrap()
        .zip(reader.read_tex_coords(0).unwrap().into_f32())
        .map(|(position, uv)| frenderer::meshes::Vertex {
            position,
            uv,
            which: 0,
        })
        .collect();
    let vert_count = verts.len();
    let fox_mesh = frend.meshes.add_mesh_group(
        &frend.gpu,
        &fox_tex,
        verts,
        (0..vert_count as u32).collect(),
        vec![frenderer::meshes::MeshEntry {
            instance_count: COUNT as u32,
            submeshes: vec![frenderer::meshes::SubmeshEntry {
                vertex_base: 0,
                indices: 0..vert_count as u32,
            }],
        }],
    );
    for trf in frend.meshes.get_meshes_mut(fox_mesh, 0) {
        *trf = Transform3D {
            translation: Vec3 {
                x: rng.gen_range(-400.0..400.0),
                y: rng.gen_range(-300.0..300.0),
                z: rng.gen_range(100.0..500.0),
            }
            .into(),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            )
            .into(),
            scale: rng.gen_range(0.5..1.0),
        };
    }
    frend.meshes.upload_meshes(&frend.gpu, fox_mesh, 0, ..);
    const DT: f32 = 1.0 / 60.0;
    const DT_FUDGE_AMOUNT: f32 = 0.0002;
    const DT_MAX: f32 = DT * 5.0;
    const TIME_SNAPS: [f32; 5] = [15.0, 30.0, 60.0, 120.0, 144.0];
    let mut acc = 0.0;
    let mut now = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        use winit::event::{Event, WindowEvent};
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // compute elapsed time since last frame
                let mut elapsed = now.elapsed().as_secs_f32();
                //println!("{elapsed}");
                // snap time to nearby vsync framerate
                TIME_SNAPS.iter().for_each(|s| {
                    if (elapsed - 1.0 / s).abs() < DT_FUDGE_AMOUNT {
                        elapsed = 1.0 / s;
                    }
                });
                // Death spiral prevention
                if elapsed > DT_MAX {
                    acc = 0.0;
                    elapsed = DT;
                }
                acc += elapsed;
                now = std::time::Instant::now();
                // While we have time to spend
                while acc >= DT {
                    // simulate a frame
                    acc -= DT;
                    // rotate every fox a random amount
                    /*
                     for trf in frend.meshes.get_meshes_mut(fox_mesh, 0) {
                         trf.rotation = (Quat::from_array(trf.rotation)
                             * Quat::from_euler(
                                 EulerRot::XYZ,
                                 rng.gen_range(0.0..(std::f32::consts::TAU * DT)),
                                 rng.gen_range(0.0..(std::f32::consts::TAU * DT)),
                                 rng.gen_range(0.0..(std::f32::consts::TAU * DT)),
                            ))
                         .into();
                     }
                     camera.translation[2] -= 100.0 * DT;
                     */

                    frend.meshes.upload_meshes(&frend.gpu, fox_mesh, 0, ..);
                    //println!("tick");
                    //update_game();
                    // camera.screen_pos[0] += 0.01;
                    input.next_frame();


                     // MOVEMENT!
                        // arrow key movement
                    if input.is_key_down(winit::event::VirtualKeyCode::Left) {
                        camera.translation[0] -= 100.0 * DT;
                    }
                    else if input.is_key_down(winit::event::VirtualKeyCode::Right) {
                        camera.translation[0] += 100.0 * DT;
                    }

                    if input.is_key_down(winit::event::VirtualKeyCode::Up) {
                        camera.translation[2] += 100.0 * DT;
                    }
                    else if input.is_key_down(winit::event::VirtualKeyCode::Down) {
                        camera.translation[2] -= 100.0 * DT;
                    }

                }
                // Render prep
                frend.meshes.set_camera(&frend.gpu, camera);
                // update sprite positions and sheet regions
                // ok now render.
                frend.render();
                window.request_redraw();
            }
            event => {
                if frend.process_window_event(&event) {
                    window.request_redraw();
                }
                input.process_input_event(&event);
            }
        }
    });
}
