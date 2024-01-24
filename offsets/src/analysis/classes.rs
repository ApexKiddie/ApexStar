use std::fmt::Write;
use pelite;
use pelite::pe64::{Va, Ptr, Pe, PeFile};
use pelite::util::CStr;
use pelite::pattern as pat;
use dataview::Pod;
use super::ident;

//----------------------------------------------------------------

pub fn print(f: &mut super::Output, bin: PeFile) {
	let classes = classes(bin);

	let _ = fmtools::write! { f.human,
		"## ClientClasses\n\n"
		for cls in &classes {
			"<details>\n"
			"<summary><code>client_class "{ident(cls.name)}"</code></summary>\n\n"
			"class_id: `"{cls.id}"`  \n"
			"sizeof: `"{cls.size:#x}"`  \n"
			"</details>\n"
		}
		"```\n\n"
	};
	let _ = fmtools::write! { f.ini,
		"[ClientClasses]\n"
		for cls in &classes {
			{ident(cls.name)}"="{cls.address:#010x}"\n"
		}
		"\n"
		"[ClientClass.Ids]\n"
		for cls in &classes {
			{ident(cls.name)}"="{cls.id}"\n"
		}
		"\n"
		"[ClientClass.Sizes]\n"
		for cls in &classes {
			{ident(cls.name)}"="{cls.size:#x}"\n"
		}
		"\n"
	};
}

//----------------------------------------------------------------

#[allow(non_snake_case)]
#[derive(Pod, Debug)]
#[repr(C)]
struct ClientClass {
	pCreateFn: Ptr,
	pCreateEventFn: Ptr,
	pNetworkName: Ptr<CStr>,
	pRecvTable: Va,
	pNext: Ptr<ClientClass>,
	ClassID: u32,
	SizeOfClass: u32,
}

//----------------------------------------------------------------

#[derive(Debug)]
pub struct Class<'a> {
	pub name: &'a str,
	pub address: u32,
	pub id: u32,
	pub size: u32,
}

pub fn classes<'a>(bin: PeFile<'a>) -> Vec<Class<'a>> {
	let mut save = [0; 8];
	let mut list = Vec::new();

	// The ClientClasses aren't fully constructed yet, find these constructors
	// ```
	// mov     rax, g_pClientClassHead
	// mov     s_ClientClass.pNext, rax
	// lea     rax, s_ClientClass
	// mov     g_pClientClassHead, rax
	// retn
	// ```
	let mut matches = bin.scanner().matches_code(pat!("488B05${'} 488905${'} 488D05${'} 488905${'} C3"));
	while matches.next(&mut save) {
		// Remove false positives
		if save[1] != save[4] || save[2] != save[3] + 0x20 {
			continue;
		}
		// Now dealing with a ClientClass
		let _ = read(bin, save[3], &mut list);
	}

	list.sort_by_key(|item| item.name);
	list
}

fn read<'a>(bin: PeFile<'a>, address: u32, list: &mut Vec<Class<'a>>) -> pelite::Result<()> {
	let client_class: &ClientClass = bin.derva(address)?;
	let name = bin.deref_c_str(client_class.pNetworkName)?.to_str()?;
	let id = client_class.ClassID;
	let size = client_class.SizeOfClass;
	list.push(Class { name, address, id, size });
	Ok(())
}
