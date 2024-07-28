use crate::{
    plot::{generate_points, setup_curve, Curve, CurvePDF, CurveWave},
    ui::{EnergyLevel, PotentialModel, PotentialModelInput},
};
use bevy::{
    color::palettes::{css::WHITE, tailwind::GRAY_500},
    prelude::*,
};
use std::f32::consts::PI;

pub fn add_plot(app: &mut App) {
    app.add_systems(Update, (setup_pdf, setup_psi));
}

fn setup_psi(
    mut commands: Commands,
    energy_level_query: Query<&EnergyLevel>,
    curve_query: Query<Entity, (With<Curve>, With<CurveWave>)>,
    model: Query<&PotentialModel>,
) {
    for m in model.iter() {
        if m.0 == PotentialModelInput::InfiniteWell {
            for e in energy_level_query.iter() {
                let points = generate_scaled_points(|x| psi(x, e));
                setup_curve(&mut commands, GRAY_500, e.0, &curve_query, points);
            }
        }
    }
}

fn setup_pdf(
    mut commands: Commands,
    energy_level_query: Query<&EnergyLevel>,
    curve_query: Query<Entity, (With<Curve>, With<CurvePDF>)>,
    model: Query<&PotentialModel>,
) {
    for m in model.iter() {
        if m.0 == PotentialModelInput::InfiniteWell {
            for e in energy_level_query.iter() {
                let points = generate_scaled_points(|x| pdf(x, e));
                setup_curve(&mut commands, WHITE, e.0, &curve_query, points);
            }
        }
    }
}

fn generate_scaled_points<F>(function: F) -> Vec<Vec2>
where
    F: Fn(f32) -> f32,
{
    let domain_points = generate_points(-10.0, 10.0, 0.02, function);
    let scaled_points: Vec<Vec2> = domain_points
        .into_iter()
        .map(|p| Vec2::new(p.x * 1e10, p.y)) // wave
        .collect();

    scaled_points
}

/// Ψ_n(x)
fn psi(x: f32, level: &EnergyLevel) -> f32 {
    let l: f32 = 2.0;
    (2.0 / l).sqrt() * ((level.0 as f32 * PI * x) / l).sin()
}

/// PDF for Ψ_n(x)
fn pdf(x: f32, level: &EnergyLevel) -> f32 {
    let psi = psi(x, level);
    psi.powi(2)
}
