extern crate std;

use std::cmp::max;

// Rectangle bin packing based on SkylineBinPack by Jukka Jylänki.
// Original: http://clb.demon.fi/files/RectangleBinPack/
pub struct BinPack {
    bin_width: i32,
    bin_height: i32,
    skyline: Vec<SkylineNode>
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect { x: x, y: y, width: width, height: height }
    }

}

struct SkylineNode {
    x: i32,
    y: i32,
    width: i32
}

impl SkylineNode {
    pub fn new(x: i32, y: i32, width: i32) -> SkylineNode {
        SkylineNode {
            x: x, y: y, width: width
        }
    }
}


impl BinPack {

    pub fn new(width: i32, height: i32) -> BinPack {

       let mut nodes: Vec<SkylineNode> = Vec::new(); 
       nodes.push(SkylineNode::new(0, 0, width));

       BinPack {
           bin_width: width,
           bin_height: height,
           skyline: nodes
       }
    }

    fn rectangle_fits(&self, sky_node_idx: uint, width: i32, height: i32) -> Option<i32> {
        let x = self.skyline[sky_node_idx].x;
        if x + width > self.bin_width {
            return None;
        }

        let mut width_left = width;
        let mut i = sky_node_idx;

        let mut y = self.skyline[sky_node_idx].y;

        while width_left > 0 {
            y = max(y, self.skyline[i].y);
            if y + height > self.bin_height {
                return None;
            }

            width_left -= self.skyline[i].width;

            i += 1;
        }

        Some(y)
    }

    fn position_new_node(&self, width: i32, height: i32) -> (Rect, Option<uint>) {
        let mut best_height = std::i32::MAX;
        let mut best_width = std::i32::MAX;
        let mut best_idx: uint;

        let mut node = Rect::new(0, 0, 0, 0);


        let mut ret_idx: Option<uint> = None;
        for i in range(0, self.skyline.len()) {
            match self.rectangle_fits(i, width, height) {
                Some(y) => {
                    if y + height < best_height ||
                       (y + height == best_height && self.skyline[i].width < best_width) {
                        best_height = y + height;
                        best_idx = i;
                        best_width = self.skyline[i].width;
                        node.x = self.skyline[i].x;
                        node.y = y;
                        node.width = width;
                        node.height = height;
                        ret_idx = Some(best_idx)
                    }
                },
                None => ()
            };
        }

        (node, ret_idx)
    }

    fn merge_skylines(&mut self) {
        
        let mut i = 0u;

        while i < self.skyline.len() - 1 {
            let node = self.skyline[i];
            let next = self.skyline[i+1];

            if node.y == next.y {
                self.skyline.get_mut(i).width += next.width;
                self.skyline.remove(i+1);
                i -= 1;
            }

            i += 1;
        }

    }

    fn add_skyline_level(&mut self, rect: Rect, index: uint) {
        
        let node = SkylineNode::new(rect.x, rect.y + rect.height, rect.width);

        self.skyline.insert(index, node);

        let mut i = index + 1;
        while i < self.skyline.len() {
            
            let cur_node = self.skyline[i];
            let prev_node = self.skyline[i-1];

            if cur_node.x < prev_node.x + prev_node.width {
                let shrink = prev_node.x + prev_node.width - cur_node.x;

                let new_node = SkylineNode::new(cur_node.x + shrink,
                                                cur_node.y,
                                                cur_node.width - shrink);

                if new_node.width <= 0 {
                    self.skyline.remove(i);
                    i -= 1;
                } else {
                   *self.skyline.get_mut(i) = new_node;
                   break;
                }

            } else {
                break;
            }

            i += 1;
        }

        self.merge_skylines();
    }

    pub fn insert(&mut self, width: i32, height: i32) -> Option<Rect> {
        let (new_node, best_index) = self.position_new_node(width, height);

        match best_index {
            Some(index) => {
                self.add_skyline_level(new_node, index); 
                Some(new_node)
            },
            None => None
        }
    }
}
