use super::{context::InterpolationCtx, Interpolater, InterpolationInstantiator};

#[derive(Debug)]
pub struct NothingInterpolation {
    ctx: InterpolationCtx,
    values: Box<[f32]>,
}

impl InterpolationInstantiator for NothingInterpolation {
    fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let ctx = InterpolationCtx::new(supporting_points);

        let values = vec![0f32; ctx.total_amount_entries()].into_boxed_slice();

        Self { ctx, values }
    }
}

impl Interpolater for NothingInterpolation {
    fn interpolate(&mut self) -> &[f32] {
        for point in self.ctx.supporting_points.iter() {
            self.values[point.x] = point.y;
        }

        &self.values
    }

    fn total_amount_entries(&self) -> usize {
        self.ctx.total_amount_entries()
    }

    fn supporting_points_mut(&mut self) -> std::slice::IterMut<'_, super::SupportingPoint> {
        self.ctx.supporting_points.iter_mut()
    }
}
