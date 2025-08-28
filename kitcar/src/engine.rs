//! Engine
use std::fmt::Debug;

use crate::Context;

/// Unified trait for all systems.
pub trait Engine<S, P, C, G>: Send + std::fmt::Debug
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    /// On startup handler
    fn startup(&mut self, _context: &mut Context<S, P, C, G>) {}

    /// On packet handler
    /// This is the central dispatcher that routes packets to the appropriate
    /// specific handler method. Users should not need to override this.
    fn packet(&mut self, context: &mut Context<S, P, C, G>, packet: &insim::Packet) {
        match packet {
            // "Both" packets
            insim::Packet::Tiny(p) => self.tiny(context, p),
            insim::Packet::Small(p) => self.small(context, p),
            insim::Packet::Scc(p) => self.scc(context, p),
            insim::Packet::Cpp(p) => self.cpp(context, p),
            insim::Packet::Reo(p) => self.reo(context, p),
            insim::Packet::Bfn(p) => self.bfn(context, p),
            insim::Packet::Rip(p) => self.rip(context, p),
            insim::Packet::Ssh(p) => self.ssh(context, p),
            insim::Packet::Axm(p) => self.axm(context, p),
            insim::Packet::Ttc(p) => self.ttc(context, p),
            insim::Packet::Mal(p) => self.mal(context, p),
            insim::Packet::Plh(p) => self.plh(context, p),
            insim::Packet::Ipb(p) => self.ipb(context, p),
            // Informational packets
            insim::Packet::Ver(p) => self.ver(context, p),
            insim::Packet::Sta(p) => self.sta(context, p),
            insim::Packet::Ism(p) => self.ism(context, p),
            insim::Packet::Mso(p) => self.mso(context, p),
            insim::Packet::Iii(p) => self.iii(context, p),
            insim::Packet::Vtn(p) => self.vtn(context, p),
            insim::Packet::Rst(p) => self.rst(context, p),
            insim::Packet::Ncn(p) => self.ncn(context, p),
            insim::Packet::Cnl(p) => self.cnl(context, p),
            insim::Packet::Cpr(p) => self.cpr(context, p),
            insim::Packet::Npl(p) => self.npl(context, p),
            insim::Packet::Plp(p) => self.plp(context, p),
            insim::Packet::Pll(p) => self.pll(context, p),
            insim::Packet::Lap(p) => self.lap(context, p),
            insim::Packet::Spx(p) => self.spx(context, p),
            insim::Packet::Pit(p) => self.pit(context, p),
            insim::Packet::Psf(p) => self.psf(context, p),
            insim::Packet::Pla(p) => self.pla(context, p),
            insim::Packet::Cch(p) => self.cch(context, p),
            insim::Packet::Pen(p) => self.pen(context, p),
            insim::Packet::Toc(p) => self.toc(context, p),
            insim::Packet::Flg(p) => self.flg(context, p),
            insim::Packet::Pfl(p) => self.pfl(context, p),
            insim::Packet::Fin(p) => self.fin(context, p),
            insim::Packet::Res(p) => self.res(context, p),
            insim::Packet::Nlp(p) => self.nlp(context, p),
            insim::Packet::Mci(p) => self.mci(context, p),
            insim::Packet::Crs(p) => self.crs(context, p),
            insim::Packet::Axi(p) => self.axi(context, p),
            insim::Packet::Axo(p) => self.axo(context, p),
            insim::Packet::Btc(p) => self.btc(context, p),
            insim::Packet::Btt(p) => self.btt(context, p),
            insim::Packet::Con(p) => self.con(context, p),
            insim::Packet::Obh(p) => self.obh(context, p),
            insim::Packet::Hlv(p) => self.hlv(context, p),
            insim::Packet::Acr(p) => self.acr(context, p),
            insim::Packet::Nci(p) => self.nci(context, p),
            insim::Packet::Uco(p) => self.uco(context, p),
            insim::Packet::Slc(p) => self.slc(context, p),
            insim::Packet::Csc(p) => self.csc(context, p),
            insim::Packet::Cim(p) => self.cim(context, p),
            insim::Packet::Aii(p) => self.aii(context, p),
            _ => {}, // Ignore all other packet types
        }
    }

    /// On shutdown handler
    fn shutdown(&mut self, _context: &mut Context<S, P, C, G>) {}

    /// On tick
    fn tick(&mut self, _context: &mut Context<S, P, C, G>) {}

    /// Handles Tiny packets
    fn tiny(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Tiny) {}

    /// Handles Small packets
    fn small(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Small) {}

    /// Handles Scc packets
    fn scc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Scc) {}

    /// Handles Cpp packets
    fn cpp(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Cpp) {}

    /// Handles Reo packets
    fn reo(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Reo) {}

    /// Handles Bfn packets
    fn bfn(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Bfn) {}

    /// Handles Rip packets
    fn rip(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Rip) {}

    /// Handles Ssh packets
    fn ssh(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ssh) {}

    /// Handles Axm packets
    fn axm(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Axm) {}

    /// Handles Ttc packets
    fn ttc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ttc) {}

    /// Handles Mal packets
    fn mal(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Mal) {}

    /// Handles Plh packets
    fn plh(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Plh) {}

    /// Handles Ipb packets
    fn ipb(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ipb) {}

    /// Handles Ver packets
    fn ver(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ver) {}

    /// Handles Sta packets
    fn sta(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Sta) {}

    /// Handles Ism packets
    fn ism(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ism) {}

    /// Handles Mso packets
    fn mso(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Mso) {}

    /// Handles Iii packets
    fn iii(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Iii) {}

    /// Handles Vtn packets
    fn vtn(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Vtn) {}

    /// Handles Rst packets
    fn rst(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Rst) {}

    /// Handles Ncn packets
    fn ncn(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Ncn) {}

    /// Handles Cnl packets
    fn cnl(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Cnl) {}

    /// Handles Cpr packets
    fn cpr(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Cpr) {}

    /// Handles Npl packets
    fn npl(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Npl) {}

    /// Handles Plp packets
    fn plp(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Plp) {}

    /// Handles Pll packets
    fn pll(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Pll) {}

    /// Handles Lap packets
    fn lap(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Lap) {}

    /// Handles Spx packets
    fn spx(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Spx) {}

    /// Handles Pit packets
    fn pit(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Pit) {}

    /// Handles Psf packets
    fn psf(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Psf) {}

    /// Handles Pla packets
    fn pla(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Pla) {}

    /// Handles Cch packets
    fn cch(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Cch) {}

    /// Handles Pen packets
    fn pen(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Pen) {}

    /// Handles Toc packets
    fn toc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Toc) {}

    /// Handles Flg packets
    fn flg(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Flg) {}

    /// Handles Pfl packets
    fn pfl(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Pfl) {}

    /// Handles Fin packets
    fn fin(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Fin) {}

    /// Handles Res packets
    fn res(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Res) {}

    /// Handles Nlp packets
    fn nlp(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Nlp) {}

    /// Handles Mci packets
    fn mci(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Mci) {}

    /// Handles Crs packets
    fn crs(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Crs) {}

    /// Handles Axi packets
    fn axi(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Axi) {}

    /// Handles Axo packets
    fn axo(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Axo) {}

    /// Handles Btc packets
    fn btc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Btc) {}

    /// Handles Btt packets
    fn btt(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Btt) {}

    /// Handles Con packets
    fn con(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Con) {}

    /// Handles Obh packets
    fn obh(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Obh) {}

    /// Handles Hlv packets
    fn hlv(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Hlv) {}

    /// Handles Acr packets
    fn acr(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Acr) {}

    /// Handles Nci packets
    fn nci(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Nci) {}

    /// Handles Uco packets
    fn uco(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Uco) {}

    /// Handles Slc packets
    fn slc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Slc) {}

    /// Handles Csc packets
    fn csc(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Csc) {}

    /// Handles Cim packets
    fn cim(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Cim) {}

    /// Handles Aii packets
    fn aii(&mut self, _context: &mut Context<S, P, C, G>, _packet: &insim::insim::Aii) {}
}
