use bevy::{color::palettes::css::GREEN, prelude::*};

use crate::{
    camera_controller::{CameraController, CameraControllerPlugin},
    ui::{
        despawn_all_entities_tu, harmonic_oscillator_button_handler,
        infinite_well_model_button_handler, listen_energy_level_ui_inputs,
        listen_potential_model_ui_inputs, listen_ui_inputs, minus_button_handler,
        plus_button_handler, setup_ui, update_energy_level_label, PlusMinusInput,
        PlusMinusInputEvent, PotentialModelInputEvent, UiInputsEvent,
    },
};

// TODO refactor such that "plot" can be used for any (for now bezier curve) plot
// i.e. move everything Ψ/PDF to a layer above

/// general plot settings
#[derive(Resource, Clone)]
pub struct PlotSettings {
    /// start of the axis domain's range
    pub domain_range_start: f32,
    /// end of the axis domain's range
    pub domain_range_end: f32,

    /// scale applied to domain coordinates to show on screen
    /// note final scale involves as well camera's transform
    /// ideally we should have direct screen settings instead (e.g. screen step) and derive scale internally
    pub screen_scale_x: f32,
    ///////////////////////////////
    /// TODO move these to domain specific settings
    /// PlotSettings should have only a screen_scale_y (analogous to screen_scale_x) field
    ///
    // scaled down y by ~max value so it fits in graph
    pub screen_scale_y_psi: f32,
    // scaled dowwn y by eye to plot together with psi
    // exact height unimportant
    pub screen_scale_y_pdf: f32,
    ///////////////////////////////
    pub ticks: TickSettings,
}

#[derive(Resource, Clone)]
pub struct TickSettings {
    /// spacing between ticks (domain units)
    pub step: f32,
}

// consider removing this.. a domain default doesn't make much sense
impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            domain_range_start: -10.0,
            domain_range_end: 10.0,
            screen_scale_x: 1.0,
            screen_scale_y_psi: 1.0,
            screen_scale_y_pdf: 1.0,
            ticks: TickSettings { step: 1.0 },
        }
    }
}

pub fn add_plot(app: &mut App) {
    app.add_event::<UiInputsEvent>()
        .add_event::<PlusMinusInputEvent>()
        .add_event::<PotentialModelInputEvent>()
        .add_plugins(CameraControllerPlugin)
        .insert_resource(PlusMinusInput::Plus)
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_light))
        .add_systems(
            Update,
            (
                setup_axes,
                draw_curve,
                listen_ui_inputs,
                update_energy_level_label,
                plus_button_handler,
                minus_button_handler,
                listen_energy_level_ui_inputs,
                infinite_well_model_button_handler,
                harmonic_oscillator_button_handler,
                listen_potential_model_ui_inputs,
            ),
        )
        .add_systems(Startup, setup_ui);
}

/// spawns bundle with bezier curve points, corresponding to data points
/// note that the bezier curve points are still in domain space
pub fn setup_curve<T>(
    commands: &mut Commands,
    color: impl Into<Color>,
    id: u32,
    curve_query: &Query<Entity, (With<Curve>, With<T>)>,
    points: Vec<Vec2>,
) where
    T: Component,
{
    despawn_all_entities_tu(commands, curve_query);

    let bezier_points = generate_path(&points, 0.3, 0.3);
    let bezier = CubicBezier::new(bezier_points).to_curve();

    commands.spawn((
        CurveWave,
        Curve {
            id,
            points: bezier,
            color: color.into(),
        },
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.01,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.4, 0.5, 0.0),
                ..default()
            },
            ..default()
        },
        CameraController::default(),
    ));
}

fn setup_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
}

/// generates bezier curve points for data points
/// basically inserts 2 control points between each consecutive point pair
// https://github.com/ivnsch/SwiftCharts/blob/c354c1945bb35a1f01b665b22474f6db28cba4a2/SwiftCharts/Views/CubicLinePathGenerator
fn generate_path(points: &[Vec2], tension1: f32, tension2: f32) -> Vec<[Vec2; 4]> {
    let mut path = vec![];

    if points.is_empty() {
        return path;
    }

    let mut p0: Vec2;
    let mut p1: Vec2;
    let mut p2: Vec2;
    let mut p3: Vec2;
    let mut tension_bezier1: f32;
    let mut tension_bezier2: f32;

    let mut previous_point1 = Vec2::new(0.0, 0.0);

    for i in 0..(points.len() - 1) {
        p1 = points[i];
        p2 = points[i + 1];

        tension_bezier1 = tension1;
        tension_bezier2 = tension2;

        if i > 0 {
            p0 = previous_point1;

            if (p2.y - p1.y) == (p2.y - p0.y) {
                tension_bezier1 = 0.0;
            }
        } else {
            tension_bezier1 = 0.0;
            p0 = p1;
        }

        if i < points.len() - 2 {
            p3 = points[i + 2];
            if (p3.y - p2.y) == (p2.y - p1.y) {
                tension_bezier2 = 0.0;
            }
        } else {
            p3 = p2;
            tension_bezier2 = 0.0;
        }

        let control_point1 = Vec2::new(
            p1.x + (p2.x - p1.x) / 3.0,
            p1.y - (p1.y - p2.y) / 3.0 - (p0.y - p1.y) * tension_bezier1,
        );

        let control_point2 = Vec2::new(
            p1.x + 2.0 * (p2.x - p1.x) / 3.0,
            p1.y - 2.0 * (p1.y - p2.y) / 3.0 + (p2.y - p3.y) * tension_bezier2,
        );

        // println!(
        //     "generated control points: {}, {}",
        //     control_point1, control_point2
        // );

        path.push([p0, control_point1, control_point2, p2]);

        previous_point1 = p2;
    }

    path
}

/// representation of domain curve
#[derive(Component)]
pub struct Curve {
    /// some identifier for debugging
    /// uniqueness needed only if used
    #[allow(dead_code)]
    id: u32,
    /// data points (domain)
    points: CubicCurve<Vec2>,
    /// color with which the curve will be displayed
    color: Color,
}

/// bevy bundle marker for Ψ curve
#[derive(Component)]
pub struct CurveWave;

/// bevy bundle marker for PDF curve
#[derive(Component)]
pub struct CurvePDF;

/// draws the curve generated in setup_curve on the screen
fn draw_curve(mut query: Query<&Curve>, mut gizmos: Gizmos) {
    for cubic_curve in &mut query {
        gizmos.linestrip_2d(cubic_curve.points.iter_positions(1000), cubic_curve.color);
    }
}

/// generates points (x, y) by evaluating function on an x
/// within [range_start, range_end], with a given step size.
pub fn generate_points<F>(range_start: f32, range_end: f32, step: f32, function: F) -> Vec<Vec2>
where
    F: Fn(f32) -> f32,
{
    let mut points = vec![];
    let mut value = range_start as f32;
    while value <= range_end as f32 {
        let x = value;
        let y = function(x);

        points.push(Vec2::new(x, y));

        value += step;
    }

    points
}

/// generates axis lines
fn setup_axes(mut gizmos: Gizmos) {
    let size = 300.0;
    let zero = 0.0;
    // x
    gizmos.line_2d(Vec2 { x: -size, y: zero }, Vec2 { x: size, y: zero }, GREEN);
    // y
    gizmos.line_2d(Vec2 { x: zero, y: -size }, Vec2 { x: zero, y: size }, GREEN);
}

/// generates axis ticks
pub fn setup_plot_ticks(gizmos: &mut Gizmos, settings: PlotSettings) {
    let domain_points = generate_points(
        settings.domain_range_start,
        settings.domain_range_end,
        settings.ticks.step,
        |x| x,
    );
    let line_height = 0.1;
    let half_line_height = line_height / 2.0;
    for point in domain_points {
        let x = point.x * settings.screen_scale_x;
        gizmos.line_2d(
            Vec2 {
                x,
                y: -half_line_height,
            },
            Vec2 {
                x,
                y: half_line_height,
            },
            GREEN,
        );
    }
}
