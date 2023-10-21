use crate::git::Git;
use crate::tmux::Tmux;

pub struct Context<'a> {
    pub tmux: Box<dyn Tmux + 'a>,
    pub git: Box<dyn Git + 'a>,
}

impl<'a> Context<'a> {
    pub fn new(tmux: impl Tmux + 'a, git: impl Git + 'a) -> Context<'a> {
        Context {
            tmux: Box::new(tmux),
            git: Box::new(git),
        }
    }
}
