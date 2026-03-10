async function onvifRequest(path: string, body: string) {
  const onvifUrl = new URL(window.location.origin);
  onvifUrl.port = '8000';

  return await fetch(`${onvifUrl.origin}${path}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/soap+xml"
    },
    body
  });
}

function envelope(body: string) {
  return `
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    ${body}
  </s:Body>
</s:Envelope>
`;
}

export function relativeMove(pan: number, tilt: number, zoom: number) {
  const xml = envelope(`
    <RelativeMove>
      <Translation>
        <PanTilt x="${pan}" y="${tilt}" />
        <Zoom x="${zoom}" />
      </Translation>
    </RelativeMove>

`);

  return onvifRequest("/onvif/PTZ", xml);
}

export function setPreset(token?: string, name?: string) {

  const xml = envelope(`
<tptz:SetPreset xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl">
  <ProfileToken>profile_1</ProfileToken>
  ${token ? `<PresetToken>${token}</PresetToken>` : ""}
  ${name ? `<PresetName>${name}</PresetName>` : ""}
</tptz:SetPreset>
`);


  return onvifRequest("/onvif/PTZ", xml);
}

export function gotoPreset(token: string) {
  const xml = envelope(`
<tptz:GotoPreset xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl">
  <ProfileToken>profile_1</ProfileToken>
  <PresetToken>${token}</PresetToken>
</tptz:GotoPreset>
`);

  return onvifRequest("/onvif/PTZ", xml);
}

function fixNamespaces(xml: string) {
  if (!xml.includes("xmlns:env")) {
    xml = xml.replace(
      "<env:Envelope>",
      `<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope"
        xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl"
        xmlns:tt="http://www.onvif.org/ver10/schema">`
    );
  }
  return xml;
}

export async function getPresets() {

  const xml = envelope(`
<GetPresets>
  <ProfileToken>profile_1</ProfileToken>
</GetPresets>
`);

  const res = await onvifRequest('/onvif/PTZ', xml);

  const text = await res.text();

  const parser = new DOMParser();

  const doc = parser.parseFromString(fixNamespaces(text), "text/xml");

  const presets = Array.from(doc.getElementsByTagName("*"))
    .filter(n => n.localName === "Preset")
    .map(p => ({
      token: p.getAttribute("token"),
      name: Array.from(p.getElementsByTagName("*"))
        .find(n => n.localName === "Name")?.textContent
    }));

  return presets;
}