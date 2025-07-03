use std::sync::Mutex;

use insim::{insim::*, net::blocking_impl::Framed, Packet};
use pyo3::{
    exceptions::{PyConnectionError, PyIOError, PyTypeError},
    prelude::*,
};

#[pyclass]
struct Client {
    inner: Mutex<Framed>,
}

#[pymethods]
impl Client {
    /// Reads one packet and converts it to a Python object.
    fn read(&self) -> PyResult<PyObject> {
        let mut client = self.inner.lock().unwrap();

        let packet = client
            .read()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        Python::with_gil(|py| -> PyResult<PyObject> {
            // FIXME: this doesnt seem right? unbind?
            Ok(packet.into_pyobject(py)?.unbind())
        })
    }

    fn write(&self, obj: &Bound<'_, PyAny>) -> PyResult<usize> {
        // This seems a bit shit.
        let packet: Packet = match obj.get_type().name()?.to_str()? {
            "Isi" => obj.extract::<PyRef<Isi>>()?.clone().into(),
            "Ver" => obj.extract::<PyRef<Ver>>()?.clone().into(),
            "Tiny" => obj.extract::<PyRef<Tiny>>()?.clone().into(),
            // "Small" => obj.extract::<PyRef<Small>>()?.clone().into(),
            "Sta" => obj.extract::<PyRef<Sta>>()?.clone().into(),
            "Sch" => obj.extract::<PyRef<Sch>>()?.clone().into(),
            "Sfp" => obj.extract::<PyRef<Sfp>>()?.clone().into(),
            "Scc" => obj.extract::<PyRef<Scc>>()?.clone().into(),
            "Cpp" => obj.extract::<PyRef<Cpp>>()?.clone().into(),
            "Ism" => obj.extract::<PyRef<Ism>>()?.clone().into(),
            "Mso" => obj.extract::<PyRef<Mso>>()?.clone().into(),
            "Iii" => obj.extract::<PyRef<Iii>>()?.clone().into(),
            "Mst" => obj.extract::<PyRef<Mst>>()?.clone().into(),
            "Mtc" => obj.extract::<PyRef<Mtc>>()?.clone().into(),
            "Mod" => obj.extract::<PyRef<Mod>>()?.clone().into(),
            "Vtn" => obj.extract::<PyRef<Vtn>>()?.clone().into(),
            "Rst" => obj.extract::<PyRef<Rst>>()?.clone().into(),
            "Ncn" => obj.extract::<PyRef<Ncn>>()?.clone().into(),
            "Cnl" => obj.extract::<PyRef<Cnl>>()?.clone().into(),
            "Cpr" => obj.extract::<PyRef<Cpr>>()?.clone().into(),
            "Npl" => obj.extract::<PyRef<Npl>>()?.clone().into(),
            "Plp" => obj.extract::<PyRef<Plp>>()?.clone().into(),
            "Pll" => obj.extract::<PyRef<Pll>>()?.clone().into(),
            "Lap" => obj.extract::<PyRef<Lap>>()?.clone().into(),
            "Spx" => obj.extract::<PyRef<Spx>>()?.clone().into(),
            "Pit" => obj.extract::<PyRef<Pit>>()?.clone().into(),
            "Psf" => obj.extract::<PyRef<Psf>>()?.clone().into(),
            "Pla" => obj.extract::<PyRef<Pla>>()?.clone().into(),
            "Cch" => obj.extract::<PyRef<Cch>>()?.clone().into(),
            "Pen" => obj.extract::<PyRef<Pen>>()?.clone().into(),
            "Toc" => obj.extract::<PyRef<Toc>>()?.clone().into(),
            "Flg" => obj.extract::<PyRef<Flg>>()?.clone().into(),
            "Pfl" => obj.extract::<PyRef<Pfl>>()?.clone().into(),
            "Fin" => obj.extract::<PyRef<Fin>>()?.clone().into(),
            "Res" => obj.extract::<PyRef<Res>>()?.clone().into(),
            "Reo" => obj.extract::<PyRef<Reo>>()?.clone().into(),
            "Nlp" => obj.extract::<PyRef<Nlp>>()?.clone().into(),
            "Mci" => obj.extract::<PyRef<Mci>>()?.clone().into(),
            "Msx" => obj.extract::<PyRef<Msx>>()?.clone().into(),
            "Msl" => obj.extract::<PyRef<Msl>>()?.clone().into(),
            "Crs" => obj.extract::<PyRef<Crs>>()?.clone().into(),
            "Bfn" => obj.extract::<PyRef<Bfn>>()?.clone().into(),
            "Axi" => obj.extract::<PyRef<Axi>>()?.clone().into(),
            "Axo" => obj.extract::<PyRef<Axo>>()?.clone().into(),
            "Btn" => obj.extract::<PyRef<Btn>>()?.clone().into(),
            // "Btc" => obj.extract::<PyRef<Btc>>()?.clone().into(),
            // "Btt" => obj.extract::<PyRef<Btt>>()?.clone().into(),
            "Rip" => obj.extract::<PyRef<Rip>>()?.clone().into(),
            "Ssh" => obj.extract::<PyRef<Ssh>>()?.clone().into(),
            // "Con" => obj.extract::<PyRef<Con>>()?.clone().into(),
            "Obh" => obj.extract::<PyRef<Obh>>()?.clone().into(),
            "Hlv" => obj.extract::<PyRef<Hlv>>()?.clone().into(),
            "Plc" => obj.extract::<PyRef<Plc>>()?.clone().into(),
            "Axm" => obj.extract::<PyRef<Axm>>()?.clone().into(),
            "Acr" => obj.extract::<PyRef<Acr>>()?.clone().into(),
            "Hcp" => obj.extract::<PyRef<Hcp>>()?.clone().into(),
            "Nci" => obj.extract::<PyRef<Nci>>()?.clone().into(),
            // "Jrr" => obj.extract::<PyRef<Jrr>>()?.clone().into(),
            // "Uco" => obj.extract::<PyRef<Uco>>()?.clone().into(),
            "Oco" => obj.extract::<PyRef<Oco>>()?.clone().into(),
            // "Ttc" => obj.extract::<PyRef<Ttc>>()?.clone().into(),
            "Slc" => obj.extract::<PyRef<Slc>>()?.clone().into(),
            "Csc" => obj.extract::<PyRef<Csc>>()?.clone().into(),
            // "Cim" => obj.extract::<PyRef<Cim>>()?.clone().into(),
            "Mal" => obj.extract::<PyRef<Mal>>()?.clone().into(),
            "Plh" => obj.extract::<PyRef<Plh>>()?.clone().into(),
            "Ipb" => obj.extract::<PyRef<Ipb>>()?.clone().into(),
            // "Aic" => obj.extract::<PyRef<Aic>>()?.clone().into(),
            "Aii" => obj.extract::<PyRef<Aii>>()?.clone().into(),
            other => {
                return Err(PyTypeError::new_err(format!(
                    "Unhandled packet type {other}"
                )))
            },
        };

        let mut client = self.inner.lock().unwrap();
        let size = client
            .write(packet)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(size)
    }
}

#[pymodule]
fn _insim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfunction]
    fn tcp(addr: &str) -> PyResult<Client> {
        let b = insim::tcp(addr);
        let inner = b
            .connect_blocking()
            .map_err(|e| PyErr::new::<PyConnectionError, _>(e.to_string()))?;
        Ok(Client {
            inner: Mutex::new(inner),
        })
    }

    m.add_class::<Isi>()?;
    m.add_class::<Ver>()?;
    m.add_class::<Tiny>()?;
    // m.add_class::<Small>()?;
    m.add_class::<Sta>()?;
    m.add_class::<Sch>()?;
    m.add_class::<Sfp>()?;
    m.add_class::<Scc>()?;
    m.add_class::<Cpp>()?;
    m.add_class::<Ism>()?;
    m.add_class::<Mso>()?;
    m.add_class::<Iii>()?;
    m.add_class::<Mst>()?;
    m.add_class::<Mtc>()?;
    m.add_class::<Mod>()?;
    m.add_class::<Vtn>()?;
    m.add_class::<Rst>()?;
    m.add_class::<Ncn>()?;
    m.add_class::<Cnl>()?;
    m.add_class::<Cpr>()?;
    m.add_class::<Npl>()?;
    m.add_class::<Plp>()?;
    m.add_class::<Pll>()?;
    m.add_class::<Lap>()?;
    m.add_class::<Spx>()?;
    m.add_class::<Pit>()?;
    m.add_class::<Psf>()?;
    m.add_class::<Pla>()?;
    m.add_class::<Cch>()?;
    m.add_class::<Pen>()?;
    m.add_class::<Toc>()?;
    m.add_class::<Flg>()?;
    m.add_class::<Pfl>()?;
    m.add_class::<Fin>()?;
    m.add_class::<Res>()?;
    m.add_class::<Reo>()?;
    m.add_class::<Nlp>()?;
    m.add_class::<Mci>()?;
    m.add_class::<Msx>()?;
    m.add_class::<Msl>()?;
    m.add_class::<Crs>()?;
    m.add_class::<Bfn>()?;
    m.add_class::<Axi>()?;
    m.add_class::<Axo>()?;
    m.add_class::<Btn>()?;
    // m.add_class::<Btc>()?;
    // m.add_class::<Btt>()?;
    m.add_class::<Rip>()?;
    m.add_class::<Ssh>()?;
    // m.add_class::<Con>()?;
    m.add_class::<Obh>()?;
    m.add_class::<Hlv>()?;
    m.add_class::<Plc>()?;
    m.add_class::<Axm>()?;
    m.add_class::<Acr>()?;
    m.add_class::<Hcp>()?;
    m.add_class::<Nci>()?;
    // m.add_class::<Jrr>()?;
    // m.add_class::<Uco>()?;
    m.add_class::<Oco>()?;
    // m.add_class::<Ttc>()?;
    m.add_class::<Slc>()?;
    m.add_class::<Csc>()?;
    // m.add_class::<Cim>()?;
    m.add_class::<Mal>()?;
    m.add_class::<Plh>()?;
    m.add_class::<Ipb>()?;
    // m.add_class::<Aic>()?;
    m.add_class::<Aii>()?;

    m.add_class::<Client>()?;
    m.add_function(wrap_pyfunction!(tcp, m)?)?;

    Ok(())
}
