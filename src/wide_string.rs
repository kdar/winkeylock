use windows_strings::PWSTR;

pub struct WideString(pub Vec<u16>);

pub trait ToWide {
  fn to_wide(&self) -> WideString;
  fn to_wide_u8_vec(&self) -> Vec<u8>;
}

impl ToWide for &str {
  fn to_wide(&self) -> WideString {
    let mut result: Vec<u16> = self.encode_utf16().collect();
    result.push(0);
    WideString(result)
  }

  fn to_wide_u8_vec(&self) -> Vec<u8> {
    let mut wide: Vec<u16> = self.encode_utf16().collect();
    wide.push(0);
    let mut result = vec![];
    for w in wide {
      for x in w.to_ne_bytes() {
        result.push(x);
      }
    }

    result
  }
}

impl ToWide for String {
  fn to_wide(&self) -> WideString {
    let mut result: Vec<u16> = self.encode_utf16().collect();
    result.push(0);
    WideString(result)
  }

  fn to_wide_u8_vec(&self) -> Vec<u8> {
    let mut wide: Vec<u16> = self.encode_utf16().collect();
    wide.push(0);
    let mut result = vec![];
    for w in wide {
      for x in w.to_ne_bytes() {
        result.push(x);
      }
    }

    result
  }
}

impl WideString {
  pub fn as_pwstr(&self) -> PWSTR {
    PWSTR(self.0.as_ptr() as *mut _)
  }
}
