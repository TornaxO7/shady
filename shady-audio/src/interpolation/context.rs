use tracing::debug;

use super::{InterpolationSection, SupportingPoint};

#[derive(Clone)]
pub struct InterpolationCtx {
    pub supporting_points: Box<[SupportingPoint]>,
    pub sections: Box<[InterpolationSection]>,
    pub bar_values: Box<[f32]>,
}

/// Constructing stuff
impl InterpolationCtx {
    pub fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let supporting_points = supporting_points
            .into_iter()
            .collect::<Vec<SupportingPoint>>()
            .into_boxed_slice();

        let sections = {
            let mut sections = Vec::new();

            if supporting_points.len() > 1 {
                for (i, supporting_point) in supporting_points[1..].iter().enumerate() {
                    let prev_supporting_point = supporting_points.get(i).unwrap();

                    let gap_size = supporting_point.x - prev_supporting_point.x - 1;
                    let there_is_a_gap = gap_size > 0;
                    if there_is_a_gap {
                        sections.push(InterpolationSection {
                            left_supporting_point_idx: i,
                            amount: gap_size,
                        });
                    }
                }
            }

            sections.into_boxed_slice()
        };

        let bar_values = {
            let amount_bars = supporting_points
                .last()
                .map(|point| point.x + 1)
                .unwrap_or(0);

            vec![0f32; amount_bars].into_boxed_slice()
        };

        let ctx = Self {
            supporting_points,
            sections,
            bar_values,
        };

        debug!("{:?}", ctx);

        ctx
    }
}

impl std::fmt::Debug for InterpolationCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sp_iter = self.supporting_points.iter().peekable();
        let mut s_iter = self.sections.iter().peekable();

        loop {
            match (sp_iter.peek(), s_iter.peek()) {
                (Some(sp), Some(s)) => {
                    if sp.x <= s.left_supporting_point_idx {
                        write!(f, "{:?}", sp)?;
                        sp_iter.next();
                    } else {
                        write!(f, "{:?}", s)?;
                        s_iter.next();
                    }
                }
                (Some(sp), None) => {
                    write!(f, "{:?}", sp)?;
                    sp_iter.next();
                }
                (None, Some(s)) => {
                    write!(f, "{:?}", s)?;
                    s_iter.next();
                }
                (None, None) => break,
            };

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_points_no_sections() {
        let ctx = InterpolationCtx::new([]);

        assert!(ctx.supporting_points.is_empty());
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn one_point_no_sections() {
        let supporting_points = [SupportingPoint { x: 0, y: 0.0 }];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn two_points_no_sections() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 1, y: 1.0 },
        ];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn two_points_one_section() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 5, y: 1.0 },
        ];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 0,
                amount: 4
            }]
        );
    }

    #[test]
    fn three_points_one_section_at_the_beginning() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 0.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 0,
                amount: 1
            }]
        );
    }

    #[test]
    fn three_points_one_section_in_the_end() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 1, y: 0.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 1,
                amount: 1
            }]
        );
    }

    #[test]
    fn three_points_two_sections() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 0.0 },
            SupportingPoint { x: 4, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(supporting_points.clone());

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[
                InterpolationSection {
                    left_supporting_point_idx: 0,
                    amount: 1
                },
                InterpolationSection {
                    left_supporting_point_idx: 1,
                    amount: 1
                }
            ]
        );
    }

    #[test]
    #[should_panic]
    fn invalid_supporting_points_ordering() {
        let supporting_points = [
            SupportingPoint { x: 1, y: 0.0 },
            SupportingPoint { x: 0, y: 0.0 },
        ];

        InterpolationCtx::new(supporting_points);
    }
}
