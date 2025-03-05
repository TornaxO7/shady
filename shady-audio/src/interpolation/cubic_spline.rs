use std::fmt::Write;

use nalgebra::{Cholesky, DMatrix, DVector, Dyn};
use tracing::debug;

use super::{Interpolater, InterpolationCtx};

type Width = usize;

#[derive(Debug, Clone)]
pub struct CubicSplineInterpolation {
    section_widths: Box<[Width]>,

    matrix: Cholesky<f32, Dyn>,
    gradients: Box<[f32]>,
    gradient_diffs: Box<[f32]>,

    gammas: DVector<f32>,
}

impl CubicSplineInterpolation {
    pub fn boxed(ctx: &InterpolationCtx) -> Box<Self> {
        Box::new(Self::new(ctx))
    }

    pub fn new(ctx: &InterpolationCtx) -> Self {
        let supporting_points = ctx.supporting_points();

        let section_widths = {
            let mut section_width = Vec::with_capacity(supporting_points.len() - 1);

            let mut prev_width = supporting_points[0].x;
            for anchor in supporting_points[1..].iter() {
                let width = anchor.x - prev_width;
                prev_width = anchor.x;

                assert!(width > 0);

                section_width.push(width);
            }

            section_width.into_boxed_slice()
        };
        let amount_sections = section_widths.len();

        let matrix = {
            let mut matrix = DMatrix::zeros(amount_sections, amount_sections);

            // add first row
            {
                let mut first_row = matrix.row_mut(0);
                first_row[0] = 2. * section_widths[0] as f32;
                first_row[1] = section_widths[0] as f32;
            }

            // add rows in between
            {
                let mut offset = 0;
                for row_idx in 1..(amount_sections - 1) {
                    let mut row = matrix.row_mut(row_idx);

                    row[offset] = section_widths[offset] as f32;
                    row[offset + 1] =
                        2. * (section_widths[offset] + section_widths[offset + 1]) as f32;
                    row[offset + 2] = section_widths[offset + 1] as f32;

                    offset += 1;
                }
            }

            // add last row
            {
                let mut last_row = matrix.row_mut(amount_sections - 1);

                last_row[amount_sections - 2] = section_widths[amount_sections - 1] as f32;
                last_row[amount_sections - 1] = 2. * section_widths[amount_sections - 1] as f32;
            }

            // just for debugging purposes
            {
                let mut dbg_matrix_msg = String::new();
                for row_idx in 0..amount_sections {
                    let row = matrix.row(row_idx);
                    write!(&mut dbg_matrix_msg, "[").unwrap();
                    for &value in row.columns_range(..amount_sections - 1) {
                        write!(&mut dbg_matrix_msg, "{value}, ").unwrap();
                    }
                    write!(&mut dbg_matrix_msg, "{}]\n", row[amount_sections - 1]).unwrap();
                }
                debug!("Matrix:\n{}", dbg_matrix_msg)
            }

            debug_assert_eq!(matrix.row(0).len(), amount_sections);
            debug_assert_eq!(matrix.column(0).len(), amount_sections);

            ((1. / 6.) * matrix)
                .cholesky()
                .expect("Hold up! Looks like my numeric knowledge isn't really numericing ;-----;")
        };

        let gradients = vec![0f32; amount_sections - 1].into_boxed_slice();
        let gradient_diffs = vec![0f32; amount_sections - 1].into_boxed_slice();

        Self {
            section_widths,
            matrix,
            gradients,
            gradient_diffs,
            gammas: DVector::zeros(amount_sections),
        }
    }
}

impl Interpolater for CubicSplineInterpolation {
    fn interpolate(&mut self, ctx: &InterpolationCtx, buffer: &mut [f32]) {
        assert!(!buffer.is_empty());
        let supporting_points = ctx.supporting_points();

        // == preparation ==
        // update gradients
        for (i, next) in supporting_points[1..].iter().enumerate() {
            let prev = &supporting_points[i];
            self.gradients[i] = (prev.y - next.y) / (prev.x as f32 - next.x as f32);
        }

        // update gradient diffs
        for (i, &next) in self.gradients[1..].iter().enumerate() {
            let prev = self.gradients[i];
            self.gradient_diffs[i] = next - prev;
        }

        // solve gamma
        let gammas = self
            .matrix
            .solve(&DVector::from_column_slice(&self.gradient_diffs));

        // == interpolation ==
        for section in ctx.sections() {
            let n = supporting_points[section.start_supporting_point + 1].x;

            let left = &supporting_points[n - 1];
            let right = &supporting_points[n];

            let prev_gamma = gammas[n - 1];
            let next_gamma = gammas[n];

            let gradient = self.gradients[n - 1];
            let section_width = self.section_widths[n - 1];

            let amount = buffer.len();
            for idx in 0..amount {
                let x = (idx + 1 + left.x) as f32;

                let interpolated_value = left.y
                    + (x - left.x as f32) * gradient
                    + ((x - left.x as f32) * (x - right.x as f32)) / (6. * section_width as f32)
                        * ((prev_gamma + 2. * next_gamma) * (x - left.x as f32)
                            - (2. * prev_gamma + next_gamma) * (x - right.x as f32));

                buffer[idx] = interpolated_value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpolation::SupportingPoint;

    use super::*;

    #[test]
    fn equidistant_supporting_points() {
        const AMOUNT_POINTS: usize = 10;
        const AMOUNT_SECTIONS: usize = AMOUNT_POINTS - 1;

        let ctx = {
            let mut ctx = InterpolationCtx::new();

            for x in 0..AMOUNT_POINTS {
                ctx.add_supporting_point(SupportingPoint { x, y: 0.0 });
            }

            ctx
        };

        let interpolator = CubicSplineInterpolation::new(&ctx);

        assert_eq!(interpolator.section_widths.as_ref(), &[1; AMOUNT_SECTIONS]);

        // check matrix
        {
            #[rustfmt::skip]
            let expected_matrix = {
                let mut matrix = DMatrix::from_row_slice(
                AMOUNT_SECTIONS,
                AMOUNT_SECTIONS,
                &[
                            2., 1., 0., 0., 0., 0., 0., 0., 0.,
                            1., 4., 1., 0., 0., 0., 0., 0., 0.,
                            0., 1., 4., 1., 0., 0., 0., 0., 0.,
                            0., 0., 1., 4., 1., 0., 0., 0., 0.,
                            0., 0., 0., 1., 4., 1., 0., 0., 0.,
                            0., 0., 0., 0., 1., 4., 1., 0., 0.,
                            0., 0., 0., 0., 0., 1., 4., 1., 0.,
                            0., 0., 0., 0., 0., 0., 1., 4., 1.,
                            0., 0., 0., 0., 0., 0., 0., 1., 2.,
                        ]);

                matrix *= 1. / 6.;
                matrix.cholesky().unwrap()
            };

            assert_eq!(interpolator.matrix.l(), expected_matrix.l());
        }
    }
}
