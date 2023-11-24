use crate::Circle;
use macroquad::prelude::*;

#[derive(Debug)]
struct QuadtreeNode {
    bounds: Rect,
    circles: Vec<Box<Circle>>,
    children: Option<Box<[Option<Box<QuadtreeNode>>; 4]>>,
}

impl QuadtreeNode {
    fn new(bounds: Rect) -> Self {
        QuadtreeNode {
            bounds,
            circles: Vec::new(),
            children: None,
        }
    }


    fn subdivide(&mut self) {
        let (x, y, w, h) = (
            self.bounds.x,
            self.bounds.y,
            self.bounds.w / 2.0,
            self.bounds.h / 2.0,
        );

        let nw = Rect::new(x, y, w, h);
        let ne = Rect::new(x + w, y, w, h);
        let sw = Rect::new(x, y + h, w, h);
        let se = Rect::new(x + w, y + h, w, h);

        self.children = Some(Box::new([
            Some(Box::new(QuadtreeNode::new(nw))),
            Some(Box::new(QuadtreeNode::new(ne))),
            Some(Box::new(QuadtreeNode::new(sw))),
            Some(Box::new(QuadtreeNode::new(se))),
        ]));
    }

    fn insert(&mut self, circle: Box<Circle>) {
        if !self.bounds.contains(vec2(circle.0, circle.1).into()) {
            return;
        }

        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.as_mut().unwrap().insert(circle.clone());
            }
        } else {
            self.circles.push(circle);

            if self.circles.len() > 5 && self.children.is_none() {
                self.subdivide();
                while let Some(c) = self.circles.pop() {
                    self.insert(c);
                }
            }
        }
    }

    fn remove(&mut self, circ: Box<Circle>) {
        if !self.bounds.contains(vec2(circ.0, circ.1).into()) {
            return;
        }

        // If the node has children, recursively remove in the children
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.as_mut().unwrap().remove(circ.clone());
            }
        } else {
            // Remove in the node's circles
            if let Some(position) = self.circles.iter().position(|c| *c == circ) {
                self.circles.remove(position);
            }

            // If the node is subdivided, remove in the children as well
            if let Some(children) = &mut self.children {
                for child in children.iter_mut() {
                    child.as_mut().unwrap().remove(circ.clone());
                }
            }
        }
    }

    fn query(&self, range: Rect, results: &mut Vec<Box<Circle>>) {
        if !self.bounds.overlaps(&range) {
            return;
        }

        for circle in &self.circles {
            if range.contains(vec2(circle.0, circle.1)) {
                results.push(circle.clone());
            }
        }

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.as_ref().unwrap().query(range, results);
            }
        }
    }
}

#[derive(Debug)]
pub struct Quadtree {
    root: QuadtreeNode,
    bounds: Rect,
}

impl Quadtree {
    pub fn new(bounds: Rect) -> Self {
        Quadtree {
            root: QuadtreeNode::new(bounds),
            bounds,
        }
    }

    pub fn get_bounds(&self) -> Rect {
        self.bounds
    }

    pub fn insert(&mut self, circle: Box<Circle>) {
        self.root.insert(circle);
    }

    pub fn replace(&mut self, old: Box<Circle>, new: Box<Circle>) {
        self.root.remove(old);
        self.root.insert(new);
    }

    pub fn query(&self, range: Rect) -> Vec<Box<Circle>> {
        let mut results = Vec::new();
        self.root.query(range, &mut results);
        results
    }
    pub fn clear(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.root = QuadtreeNode::new(bounds);
    }
}
