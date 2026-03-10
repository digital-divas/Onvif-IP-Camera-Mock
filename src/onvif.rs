use axum::{
    Json, Router,
    body::Bytes,
    extract::{OriginalUri, State},
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use roxmltree::Document;

use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info_span;

#[derive(Debug, Clone)]
pub struct Preset {
    pub token: String,
    pub name: String,
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
}

#[derive(Debug)]
pub struct CameraState {
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
    pub presets: Vec<Preset>,
}

pub type SharedCameraState = Arc<Mutex<CameraState>>;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> impl IntoResponse {
    return (StatusCode::OK, Json(HealthResponse { status: "ok" }));
}

async fn fallback_handler(
    method: Method,
    uri: OriginalUri,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    eprintln!("--- INCOMING REQUEST ---");
    eprintln!("Method: {}", method);
    eprintln!("URI: {}", uri.0);

    eprintln!("Headers:");
    for (k, v) in headers.iter() {
        eprintln!("  {}: {:?}", k, v);
    }

    if !body.is_empty() {
        eprintln!("Body:\n{}", String::from_utf8_lossy(&body));
    }

    eprintln!("------------------------");

    return StatusCode::NOT_FOUND;
}

fn get_services_response() -> String {
    let body = r#"
    <?xml version='1.0' encoding='UTF-8'?>
        <env:Envelope                                      ><env:Body><tds:GetServicesResponse><tds:Service><tds:Namespace>http://www.onvif.org/ver10/device/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/device_service</tds:XAddr>
        <tds:Capabilities><tds:Capabilities><tds:Network IPFilter='true' ZeroConfiguration='true' IPVersion6='true' DynDNS='true' Dot11Configuration='false' Dot1XConfigurations='0' HostnameFromDHCP='true' NTP='1' DHCPv6='true'></tds:Network>
        <tds:Security TLS1.0='true' TLS1.1='true' TLS1.2='true' OnboardKeyGeneration='false' AccessPolicyConfig='false' DefaultAccessPolicy='true' Dot1X='false' RemoteUserHandling='false' X.509Token='false' SAMLToken='false' KerberosToken='false' UsernameToken='true' HttpDigest='true' RELToken='false' SupportedEAPMethods='0' MaxUsers='32' MaxUserNameLength='32' MaxPasswordLength='16'></tds:Security>
        <tds:System DiscoveryResolve='false' DiscoveryBye='true' RemoteDiscovery='false' SystemBackup='false' SystemLogging='true' FirmwareUpgrade='true' HttpFirmwareUpgrade='true' HttpSystemBackup='false' HttpSystemLogging='false' HttpSupportInformation='false' StorageConfiguration='true' MaxStorageConfigurations='8' GeoLocationEntries='2' AutoGeo='Location'></tds:System>
        </tds:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>18</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/media/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Media</tds:XAddr>
        <tds:Capabilities><trt:Capabilities SnapshotUri='true' Rotation='false' VideoSourceMode='false' OSD='true'><trt:ProfileCapabilities MaximumNumberOfProfiles='20'></trt:ProfileCapabilities>
        <trt:StreamingCapabilities RTPMulticast='true' RTP_TCP='true' RTP_RTSP_TCP='true' NonAggregateControl='false'></trt:StreamingCapabilities>
        </trt:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>2</tt:Major>
        <tt:Minor>60</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/events/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Events</tds:XAddr>
        <tds:Capabilities><tev:Capabilities WSSubscriptionPolicySupport='true' WSPullPointSupport='true' WSPausableSubscriptionManagerInterfaceSupport='false' MaxNotificationProducers='10' MaxPullPoints='10'></tev:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>2</tt:Major>
        <tt:Minor>60</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver20/ptz/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/PTZ</tds:XAddr>
        <tds:Capabilities><tptz:Capabilities EFlip='false' Reverse='false' GetCompatibleConfigurations='true' MoveStatus='true' StatusPosition='true'></tptz:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver20/imaging/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Imaging</tds:XAddr>
        <tds:Capabilities><timg:Capabilities ImageStabilization='false'></timg:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>6</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/deviceIO/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/DeviceIO</tds:XAddr>
        <tds:Capabilities><tmd:Capabilities VideoSources='1' VideoOutputs='0' AudioSources='1' AudioOutputs='1' RelayOutputs='2' DigitalInputs='7' SerialPorts='1' DigitalInputOptions='true'></tmd:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver20/analytics/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Analytics</tds:XAddr>
        <tds:Capabilities><tan:Capabilities RuleSupport='true' AnalyticsModuleSupport='true' CellBasedSceneDescriptionSupported='true' RuleOptionsSupported='true' AnalyticsModuleOptionsSupported='false'/>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/recording/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Recording</tds:XAddr>
        <tds:Capabilities><trc:Capabilities DynamicRecordings='false' DynamicTracks='false' Encoding='G711 G726 AAC H264 H265 JPEG' MaxRate='16384' MaxTotalRate='16384' MaxRecordings='1' MaxRecordingJobs='1' Options='true'></trc:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/search/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/SearchRecording</tds:XAddr>
        <tds:Capabilities><tse:Capabilities MetadataSearch='false' GeneralStartEvents='false'></tse:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>2</tt:Major>
        <tt:Minor>42</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver10/replay/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Replay</tds:XAddr>
        <tds:Capabilities><trp:Capabilities ReversePlayback='false' SessionTimeoutRange='1 60' RTP_RTSP_TCP='true'></trp:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>2</tt:Major>
        <tt:Minor>21</tt:Minor>
        </tds:Version>
        </tds:Service>
        <tds:Service><tds:Namespace>http://www.onvif.org/ver20/media/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.20.166:9296/onvif/Media2</tds:XAddr>
        <tds:Capabilities><tr2:Capabilities SnapshotUri='true' Rotation='false' VideoSourceMode='false' OSD='true' Mask='true' SourceMask='true'><tr2:ProfileCapabilities MaximumNumberOfProfiles='20' ConfigurationsSupported='VideoSource VideoEncoder AudioSource AudioEncoder AudioOutput AudioDecoder Metadata Analytics PTZ'></tr2:ProfileCapabilities>
        <tr2:StreamingCapabilities RTSPStreaming='true' RTPMulticast='true' RTP_RTSP_TCP='true' NonAggregateControl='false' AutoStartMulticast='true'></tr2:StreamingCapabilities>
        </tr2:Capabilities>
        </tds:Capabilities>
        <tds:Version><tt:Major>16</tt:Major>
        <tt:Minor>12</tt:Minor>
        </tds:Version>
        </tds:Service>
        </tds:GetServicesResponse>
        </env:Body>
        </env:Envelope>`;
    "#;

    body.to_string()
}

fn get_system_date_and_time_response() -> String {
    let body = r#"
        <?xml version='1.0' encoding='UTF-8'?>
        <env:Envelope xmlns:env='http://www.w3.org/2003/05/soap-envelope' xmlns:soapenc='http://www.w3.org/2003/05/soap-encoding' xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance' xmlns:xs='http://www.w3.org/2001/XMLSchema' xmlns:tt='http://www.onvif.org/ver10/schema' xmlns:tds='http://www.onvif.org/ver10/device/wsdl' xmlns:trt='http://www.onvif.org/ver10/media/wsdl' xmlns:timg='http://www.onvif.org/ver20/imaging/wsdl' xmlns:tev='http://www.onvif.org/ver10/events/wsdl' xmlns:tptz='http://www.onvif.org/ver20/ptz/wsdl' xmlns:tan='http://www.onvif.org/ver20/analytics/wsdl' xmlns:tst='http://www.onvif.org/ver10/storage/wsdl' xmlns:ter='http://www.onvif.org/ver10/error' xmlns:dn='http://www.onvif.org/ver10/network/wsdl' xmlns:tns1='http://www.onvif.org/ver10/topics' xmlns:tmd='http://www.onvif.org/ver10/deviceIO/wsdl' xmlns:wsdl='http://schemas.xmlsoap.org/wsdl' xmlns:wsoap12='http://schemas.xmlsoap.org/wsdl/soap12' xmlns:http='http://schemas.xmlsoap.org/wsdl/http' xmlns:d='http://schemas.xmlsoap.org/ws/2005/04/discovery' xmlns:wsadis='http://schemas.xmlsoap.org/ws/2004/08/addressing' xmlns:wsnt='http://docs.oasis-open.org/wsn/b-2' xmlns:wsa='http://www.w3.org/2005/08/addressing' xmlns:wstop='http://docs.oasis-open.org/wsn/t-1' xmlns:wsrf-bf='http://docs.oasis-open.org/wsrf/bf-2' xmlns:wsntw='http://docs.oasis-open.org/wsn/bw-2' xmlns:wsrf-rw='http://docs.oasis-open.org/wsrf/rw-2' xmlns:wsaw='http://www.w3.org/2006/05/addressing/wsdl' xmlns:wsrf-r='http://docs.oasis-open.org/wsrf/r-2' xmlns:trc='http://www.onvif.org/ver10/recording/wsdl' xmlns:tse='http://www.onvif.org/ver10/search/wsdl' xmlns:trp='http://www.onvif.org/ver10/replay/wsdl' xmlns:tnshik='http://www.hikvision.com/2011/event/topics' xmlns:hikwsd='http://www.onvifext.com/onvif/ext/ver10/wsdl' xmlns:hikxsd='http://www.onvifext.com/onvif/ext/ver10/schema' xmlns:tas='http://www.onvif.org/ver10/advancedsecurity/wsdl' xmlns:tr2='http://www.onvif.org/ver20/media/wsdl' xmlns:axt='http://www.onvif.org/ver20/analytics'>
            <env:Body>
                <tds:GetSystemDateAndTimeResponse>
                    <tds:SystemDateAndTime>
                        <tt:DateTimeType>NTP</tt:DateTimeType>
                        <tt:DaylightSavings>false</tt:DaylightSavings>
                        <tt:TimeZone>
                            <tt:TZ>CST+3:00:00</tt:TZ>
                        </tt:TimeZone>
                        <tt:UTCDateTime>
                            <tt:Time>
                                <tt:Hour>18</tt:Hour>
                                <tt:Minute>49</tt:Minute>
                                <tt:Second>55</tt:Second>
                            </tt:Time>
                            <tt:Date>
                                <tt:Year>2023</tt:Year>
                                <tt:Month>12</tt:Month>
                                <tt:Day>1</tt:Day>
                            </tt:Date>
                        </tt:UTCDateTime>
                        <tt:LocalDateTime>
                            <tt:Time>
                                <tt:Hour>15</tt:Hour>
                                <tt:Minute>49</tt:Minute>
                                <tt:Second>55</tt:Second>
                            </tt:Time>
                            <tt:Date>
                                <tt:Year>2023</tt:Year>
                                <tt:Month>12</tt:Month>
                                <tt:Day>1</tt:Day>
                            </tt:Date>
                        </tt:LocalDateTime>
                    </tds:SystemDateAndTime>
                </tds:GetSystemDateAndTimeResponse>
            </env:Body>
        </env:Envelope>
    "#;

    body.to_string()
}

fn detect_onvif_op(xml: &str) -> Option<String> {
    let doc = Document::parse(xml).ok()?;

    let body = doc.descendants().find(|n| n.has_tag_name("Body"))?;
    let op = body.children().find(|n| n.is_element())?;
    return Some(op.tag_name().name().to_string());
}

fn get_response_for_action(action: String) -> String {
    let empty_string = "";

    if action.ends_with("GetSystemDateAndTime") {
        return get_system_date_and_time_response();
    }
    if action.ends_with("GetServices") {
        return get_services_response();
    }

    return empty_string.to_string();
}

async fn device_server(body: Bytes) -> impl IntoResponse {
    let xml = String::from_utf8_lossy(&body);

    let action = detect_onvif_op(&xml).unwrap_or_default();

    let response = get_response_for_action(action);

    return (
        StatusCode::OK,
        [("Content-Type", "application/soap+xml; charset=utf-8")],
        response,
    );
}

fn get_video_sources_response() -> String {
    let body = r#"
        <?xml version='1.0' encoding='UTF-8'?>
        <env:Envelope>
        <env:Body>
        <trt:GetVideoSourcesResponse>
        <trt:VideoSources token='VideoSource_1'>
        <tt:Framerate>24</tt:Framerate>
        <tt:Resolution><tt:Width>3840</tt:Width>
        <tt:Height>2160</tt:Height>
        </tt:Resolution>
        <tt:Imaging><tt:BacklightCompensation><tt:Mode>OFF</tt:Mode>
        <tt:Level>0</tt:Level>
        </tt:BacklightCompensation>
        <tt:Brightness>50</tt:Brightness>
        <tt:ColorSaturation>50</tt:ColorSaturation>
        <tt:Contrast>50</tt:Contrast>
        <tt:Exposure><tt:Mode>AUTO</tt:Mode>
        <tt:Priority>LowNoise</tt:Priority>
        <tt:Window bottom='0' top='0' right='0' left='0'/>
        <tt:MinExposureTime>33</tt:MinExposureTime>
        <tt:MaxExposureTime>10000</tt:MaxExposureTime>
        <tt:MinGain>0</tt:MinGain>
        <tt:MaxGain>63</tt:MaxGain>
        <tt:MinIris>0</tt:MinIris>
        <tt:MaxIris>100</tt:MaxIris>
        <tt:ExposureTime>33333</tt:ExposureTime>
        <tt:Gain>0</tt:Gain>
        <tt:Iris>0</tt:Iris>
        </tt:Exposure>
        <tt:Focus><tt:AutoFocusMode>AUTO</tt:AutoFocusMode>
        <tt:DefaultSpeed>1</tt:DefaultSpeed>
        <tt:NearLimit>600</tt:NearLimit>
        <tt:FarLimit>0</tt:FarLimit>
        </tt:Focus>
        <tt:IrCutFilter>AUTO</tt:IrCutFilter>
        <tt:Sharpness>50</tt:Sharpness>
        <tt:WideDynamicRange><tt:Mode>OFF</tt:Mode>
        <tt:Level>50</tt:Level>
        </tt:WideDynamicRange>
        <tt:WhiteBalance><tt:Mode>AUTO</tt:Mode>
        <tt:CrGain>0</tt:CrGain>
        <tt:CbGain>0</tt:CbGain>
        </tt:WhiteBalance>
        </tt:Imaging>
        </trt:VideoSources>
        </trt:GetVideoSourcesResponse>
        </env:Body>
        </env:Envelope>"#;

    body.to_string()
}

async fn media() -> impl IntoResponse {
    let response = get_video_sources_response();
    return (
        StatusCode::OK,
        [("Content-Type", "application/soap+xml; charset=utf-8")],
        response,
    );
}

fn get_profiles_response() -> String {
    let body = r#"
            <?xml version='1.0' encoding='UTF-8'?>
        <env:Envelope                                      ><env:Body><tr2:GetProfilesResponse><tr2:Profiles token='Profile_1' fixed='true'><tr2:Name>mainStream</tr2:Name>
        <tr2:Configurations><tr2:VideoSource token='VideoSourceToken'><tt:Name>VideoSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>VideoSource_1</tt:SourceToken>
        <tt:Bounds x='0' y='0' width='3840' height='2160'></tt:Bounds>
        </tr2:VideoSource>
        <tr2:AudioSource token='AudioSourceConfigToken'><tt:Name>AudioSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>AudioSourceChannel</tt:SourceToken>
        </tr2:AudioSource>
        <tr2:VideoEncoder token='VideoEncoderToken_1' GovLength='50' Profile='High'><tt:Name>VideoEncoder_1</tt:Name>
        <tt:UseCount>1</tt:UseCount>
        <tt:Encoding>H264</tt:Encoding>
        <tt:Resolution><tt:Width>1920</tt:Width>
        <tt:Height>1080</tt:Height>
        </tt:Resolution>
        <tt:RateControl ConstantBitRate='false'><tt:FrameRateLimit>24.000000</tt:FrameRateLimit>
        <tt:BitrateLimit>12288</tt:BitrateLimit>
        </tt:RateControl>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8860</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Quality>3.000000</tt:Quality>
        </tr2:VideoEncoder>
        <tr2:AudioEncoder token='MainAudioEncoderToken'><tt:Name>AudioEncoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:Encoding>PCMU</tt:Encoding>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8862</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Bitrate>64</tt:Bitrate>
        <tt:SampleRate>8</tt:SampleRate>
        </tr2:AudioEncoder>
        <tr2:Analytics token='VideoAnalyticsToken'><tt:Name>VideoAnalyticsName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:AnalyticsEngineConfiguration><tt:AnalyticsModule Name='MyCellMotionModule' Type='tt:CellMotionEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Layout'><tt:CellLayout Columns='22' Rows='15'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.090909' y='0.133333'/>
        </tt:Transformation>
        </tt:CellLayout>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        <tt:AnalyticsModule Name='MyTamperDetecModule' Type='hikxsd:TamperEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Transformation'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.002841' y='0.004167'/>
        </tt:Transformation>
        </tt:ElementItem>
        <tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='480'/>
        <tt:Point x='704' y='480'/>
        <tt:Point x='704' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        </tt:AnalyticsEngineConfiguration>
        <tt:RuleEngineConfiguration><tt:Rule Name='MyMotionDetectorRule' Type='tt:CellMotionDetector'><tt:Parameters><tt:SimpleItem Name='MinCount' Value='5'/>
        <tt:SimpleItem Name='AlarmOnDelay' Value='1000'/>
        <tt:SimpleItem Name='AlarmOffDelay' Value='1000'/>
        <tt:SimpleItem Name='ActiveCells' Value='1wA='/>
        </tt:Parameters>
        </tt:Rule>
        <tt:Rule Name='MyTamperDetectorRule' Type='hikxsd:TamperDetector'><tt:Parameters><tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:Rule>
        </tt:RuleEngineConfiguration>
        </tr2:Analytics>
        <tr2:PTZ token='PTZToken'><tt:Name>PTZ</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:NodeToken>PTZNODETOKEN</tt:NodeToken>
        <tt:DefaultAbsolutePantTiltPositionSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:DefaultAbsolutePantTiltPositionSpace>
        <tt:DefaultAbsoluteZoomPositionSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:DefaultAbsoluteZoomPositionSpace>
        <tt:DefaultRelativePanTiltTranslationSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/TranslationGenericSpace</tt:DefaultRelativePanTiltTranslationSpace>
        <tt:DefaultRelativeZoomTranslationSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/TranslationGenericSpace</tt:DefaultRelativeZoomTranslationSpace>
        <tt:DefaultContinuousPanTiltVelocitySpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/VelocityGenericSpace</tt:DefaultContinuousPanTiltVelocitySpace>
        <tt:DefaultContinuousZoomVelocitySpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/VelocityGenericSpace</tt:DefaultContinuousZoomVelocitySpace>
        <tt:DefaultPTZSpeed><tt:PanTilt x='0.100000' y='0.100000' space='http://www.onvif.org/ver10/tptz/PanTiltSpaces/GenericSpeedSpace'/>
        <tt:Zoom x='1.000000' space='http://www.onvif.org/ver10/tptz/ZoomSpaces/ZoomGenericSpeedSpace'/>
        </tt:DefaultPTZSpeed>
        <tt:DefaultPTZTimeout>PT300S</tt:DefaultPTZTimeout>
        <tt:PanTiltLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        <tt:YRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:YRange>
        </tt:Range>
        </tt:PanTiltLimits>
        <tt:ZoomLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>0.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        </tt:Range>
        </tt:ZoomLimits>
        </tr2:PTZ>
        <tr2:AudioOutput token='AudioOutputConfigToken'><tt:Name>AudioOutputConfigName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:OutputToken>AudioOutputToken</tt:OutputToken>
        <tt:SendPrimacy>www.onvif.org/ver20/HalfDuplex/Auto</tt:SendPrimacy>
        <tt:OutputLevel>100</tt:OutputLevel>
        </tr2:AudioOutput>
        <tr2:AudioDecoder token='AudioDecoderConfigToken'><tt:Name>AudioDecoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        </tr2:AudioDecoder>
        </tr2:Configurations>
        </tr2:Profiles>
        <tr2:Profiles token='Profile_2' fixed='true'><tr2:Name>subStream</tr2:Name>
        <tr2:Configurations><tr2:VideoSource token='VideoSourceToken'><tt:Name>VideoSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>VideoSource_1</tt:SourceToken>
        <tt:Bounds x='0' y='0' width='3840' height='2160'></tt:Bounds>
        </tr2:VideoSource>
        <tr2:AudioSource token='AudioSourceConfigToken'><tt:Name>AudioSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>AudioSourceChannel</tt:SourceToken>
        </tr2:AudioSource>
        <tr2:VideoEncoder token='VideoEncoderToken_2' GovLength='50'><tt:Name>VideoEncoder_2</tt:Name>
        <tt:UseCount>1</tt:UseCount>
        <tt:Encoding>JPEG</tt:Encoding>
        <tt:Resolution><tt:Width>640</tt:Width>
        <tt:Height>480</tt:Height>
        </tt:Resolution>
        <tt:RateControl ConstantBitRate='false'><tt:FrameRateLimit>24.000000</tt:FrameRateLimit>
        <tt:BitrateLimit>1024</tt:BitrateLimit>
        </tt:RateControl>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8866</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Quality>3.000000</tt:Quality>
        </tr2:VideoEncoder>
        <tr2:AudioEncoder token='MainAudioEncoderToken'><tt:Name>AudioEncoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:Encoding>PCMU</tt:Encoding>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8862</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Bitrate>64</tt:Bitrate>
        <tt:SampleRate>8</tt:SampleRate>
        </tr2:AudioEncoder>
        <tr2:Analytics token='VideoAnalyticsToken'><tt:Name>VideoAnalyticsName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:AnalyticsEngineConfiguration><tt:AnalyticsModule Name='MyCellMotionModule' Type='tt:CellMotionEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Layout'><tt:CellLayout Columns='22' Rows='15'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.090909' y='0.133333'/>
        </tt:Transformation>
        </tt:CellLayout>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        <tt:AnalyticsModule Name='MyTamperDetecModule' Type='hikxsd:TamperEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Transformation'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.002841' y='0.004167'/>
        </tt:Transformation>
        </tt:ElementItem>
        <tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='480'/>
        <tt:Point x='704' y='480'/>
        <tt:Point x='704' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        </tt:AnalyticsEngineConfiguration>
        <tt:RuleEngineConfiguration><tt:Rule Name='MyMotionDetectorRule' Type='tt:CellMotionDetector'><tt:Parameters><tt:SimpleItem Name='MinCount' Value='5'/>
        <tt:SimpleItem Name='AlarmOnDelay' Value='1000'/>
        <tt:SimpleItem Name='AlarmOffDelay' Value='1000'/>
        <tt:SimpleItem Name='ActiveCells' Value='1wA='/>
        </tt:Parameters>
        </tt:Rule>
        <tt:Rule Name='MyTamperDetectorRule' Type='hikxsd:TamperDetector'><tt:Parameters><tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:Rule>
        </tt:RuleEngineConfiguration>
        </tr2:Analytics>
        <tr2:PTZ token='PTZToken'><tt:Name>PTZ</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:NodeToken>PTZNODETOKEN</tt:NodeToken>
        <tt:DefaultAbsolutePantTiltPositionSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:DefaultAbsolutePantTiltPositionSpace>
        <tt:DefaultAbsoluteZoomPositionSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:DefaultAbsoluteZoomPositionSpace>
        <tt:DefaultRelativePanTiltTranslationSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/TranslationGenericSpace</tt:DefaultRelativePanTiltTranslationSpace>
        <tt:DefaultRelativeZoomTranslationSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/TranslationGenericSpace</tt:DefaultRelativeZoomTranslationSpace>
        <tt:DefaultContinuousPanTiltVelocitySpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/VelocityGenericSpace</tt:DefaultContinuousPanTiltVelocitySpace>
        <tt:DefaultContinuousZoomVelocitySpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/VelocityGenericSpace</tt:DefaultContinuousZoomVelocitySpace>
        <tt:DefaultPTZSpeed><tt:PanTilt x='0.100000' y='0.100000' space='http://www.onvif.org/ver10/tptz/PanTiltSpaces/GenericSpeedSpace'/>
        <tt:Zoom x='1.000000' space='http://www.onvif.org/ver10/tptz/ZoomSpaces/ZoomGenericSpeedSpace'/>
        </tt:DefaultPTZSpeed>
        <tt:DefaultPTZTimeout>PT300S</tt:DefaultPTZTimeout>
        <tt:PanTiltLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        <tt:YRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:YRange>
        </tt:Range>
        </tt:PanTiltLimits>
        <tt:ZoomLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>0.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        </tt:Range>
        </tt:ZoomLimits>
        </tr2:PTZ>
        <tr2:AudioOutput token='AudioOutputConfigToken'><tt:Name>AudioOutputConfigName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:OutputToken>AudioOutputToken</tt:OutputToken>
        <tt:SendPrimacy>www.onvif.org/ver20/HalfDuplex/Auto</tt:SendPrimacy>
        <tt:OutputLevel>100</tt:OutputLevel>
        </tr2:AudioOutput>
        <tr2:AudioDecoder token='AudioDecoderConfigToken'><tt:Name>AudioDecoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        </tr2:AudioDecoder>
        </tr2:Configurations>
        </tr2:Profiles>
        <tr2:Profiles token='Profile_3' fixed='true'><tr2:Name>thirdStream</tr2:Name>
        <tr2:Configurations><tr2:VideoSource token='VideoSourceToken'><tt:Name>VideoSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>VideoSource_1</tt:SourceToken>
        <tt:Bounds x='0' y='0' width='3840' height='2160'></tt:Bounds>
        </tr2:VideoSource>
        <tr2:AudioSource token='AudioSourceConfigToken'><tt:Name>AudioSourceConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:SourceToken>AudioSourceChannel</tt:SourceToken>
        </tr2:AudioSource>
        <tr2:VideoEncoder token='VideoEncoderToken_3' GovLength='50' Profile='High'><tt:Name>VideoEncoder_3</tt:Name>
        <tt:UseCount>1</tt:UseCount>
        <tt:Encoding>H264</tt:Encoding>
        <tt:Resolution><tt:Width>1920</tt:Width>
        <tt:Height>1080</tt:Height>
        </tt:Resolution>
        <tt:RateControl ConstantBitRate='false'><tt:FrameRateLimit>24.000000</tt:FrameRateLimit>
        <tt:BitrateLimit>1024</tt:BitrateLimit>
        </tt:RateControl>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8872</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Quality>3.000000</tt:Quality>
        </tr2:VideoEncoder>
        <tr2:AudioEncoder token='MainAudioEncoderToken'><tt:Name>AudioEncoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:Encoding>PCMU</tt:Encoding>
        <tt:Multicast><tt:Address><tt:Type>IPv4</tt:Type>
        <tt:IPv4Address>0.0.0.0</tt:IPv4Address>
        </tt:Address>
        <tt:Port>8862</tt:Port>
        <tt:TTL>128</tt:TTL>
        <tt:AutoStart>false</tt:AutoStart>
        </tt:Multicast>
        <tt:Bitrate>64</tt:Bitrate>
        <tt:SampleRate>8</tt:SampleRate>
        </tr2:AudioEncoder>
        <tr2:Analytics token='VideoAnalyticsToken'><tt:Name>VideoAnalyticsName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:AnalyticsEngineConfiguration><tt:AnalyticsModule Name='MyCellMotionModule' Type='tt:CellMotionEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Layout'><tt:CellLayout Columns='22' Rows='15'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.090909' y='0.133333'/>
        </tt:Transformation>
        </tt:CellLayout>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        <tt:AnalyticsModule Name='MyTamperDetecModule' Type='hikxsd:TamperEngine'><tt:Parameters><tt:SimpleItem Name='Sensitivity' Value='0'/>
        <tt:ElementItem Name='Transformation'><tt:Transformation><tt:Translate x='-1.000000' y='-1.000000'/>
        <tt:Scale x='0.002841' y='0.004167'/>
        </tt:Transformation>
        </tt:ElementItem>
        <tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='480'/>
        <tt:Point x='704' y='480'/>
        <tt:Point x='704' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:AnalyticsModule>
        </tt:AnalyticsEngineConfiguration>
        <tt:RuleEngineConfiguration><tt:Rule Name='MyMotionDetectorRule' Type='tt:CellMotionDetector'><tt:Parameters><tt:SimpleItem Name='MinCount' Value='5'/>
        <tt:SimpleItem Name='AlarmOnDelay' Value='1000'/>
        <tt:SimpleItem Name='AlarmOffDelay' Value='1000'/>
        <tt:SimpleItem Name='ActiveCells' Value='1wA='/>
        </tt:Parameters>
        </tt:Rule>
        <tt:Rule Name='MyTamperDetectorRule' Type='hikxsd:TamperDetector'><tt:Parameters><tt:ElementItem Name='Field'><tt:PolygonConfiguration><tt:Polygon><tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        <tt:Point x='0' y='0'/>
        </tt:Polygon>
        </tt:PolygonConfiguration>
        </tt:ElementItem>
        </tt:Parameters>
        </tt:Rule>
        </tt:RuleEngineConfiguration>
        </tr2:Analytics>
        <tr2:PTZ token='PTZToken'><tt:Name>PTZ</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:NodeToken>PTZNODETOKEN</tt:NodeToken>
        <tt:DefaultAbsolutePantTiltPositionSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:DefaultAbsolutePantTiltPositionSpace>
        <tt:DefaultAbsoluteZoomPositionSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:DefaultAbsoluteZoomPositionSpace>
        <tt:DefaultRelativePanTiltTranslationSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/TranslationGenericSpace</tt:DefaultRelativePanTiltTranslationSpace>
        <tt:DefaultRelativeZoomTranslationSpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/TranslationGenericSpace</tt:DefaultRelativeZoomTranslationSpace>
        <tt:DefaultContinuousPanTiltVelocitySpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/VelocityGenericSpace</tt:DefaultContinuousPanTiltVelocitySpace>
        <tt:DefaultContinuousZoomVelocitySpace>http://www.onvif.org/ver10/tptz/ZoomSpaces/VelocityGenericSpace</tt:DefaultContinuousZoomVelocitySpace>
        <tt:DefaultPTZSpeed><tt:PanTilt x='0.100000' y='0.100000' space='http://www.onvif.org/ver10/tptz/PanTiltSpaces/GenericSpeedSpace'/>
        <tt:Zoom x='1.000000' space='http://www.onvif.org/ver10/tptz/ZoomSpaces/ZoomGenericSpeedSpace'/>
        </tt:DefaultPTZSpeed>
        <tt:DefaultPTZTimeout>PT300S</tt:DefaultPTZTimeout>
        <tt:PanTiltLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        <tt:YRange><tt:Min>-1.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:YRange>
        </tt:Range>
        </tt:PanTiltLimits>
        <tt:ZoomLimits><tt:Range><tt:URI>http://www.onvif.org/ver10/tptz/ZoomSpaces/PositionGenericSpace</tt:URI>
        <tt:XRange><tt:Min>0.000000</tt:Min>
        <tt:Max>1.000000</tt:Max>
        </tt:XRange>
        </tt:Range>
        </tt:ZoomLimits>
        </tr2:PTZ>
        <tr2:AudioOutput token='AudioOutputConfigToken'><tt:Name>AudioOutputConfigName</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        <tt:OutputToken>AudioOutputToken</tt:OutputToken>
        <tt:SendPrimacy>www.onvif.org/ver20/HalfDuplex/Auto</tt:SendPrimacy>
        <tt:OutputLevel>100</tt:OutputLevel>
        </tr2:AudioOutput>
        <tr2:AudioDecoder token='AudioDecoderConfigToken'><tt:Name>AudioDecoderConfig</tt:Name>
        <tt:UseCount>3</tt:UseCount>
        </tr2:AudioDecoder>
        </tr2:Configurations>
        </tr2:Profiles>
        </tr2:GetProfilesResponse>
        </env:Body>
        </env:Envelope>
    "#;
    body.to_string()
}

async fn media2() -> impl IntoResponse {
    let response = get_profiles_response();
    return (
        StatusCode::OK,
        [("Content-Type", "application/soap+xml; charset=utf-8")],
        response,
    );
}

fn default_ptz_success_response() -> String {
    let response =
        "<?xml version='1.0' encoding='UTF-8'?><env:Envelope><env:Body></env:Body></env:Envelope>";
    response.to_string()
}

fn parse_set_preset_params(xml: &str) -> Option<(String, String)> {
    let doc: Document<'_> = Document::parse(xml).ok()?;

    let body = doc.descendants().find(|n| n.has_tag_name("Body"))?;
    let set_preset = body.descendants().find(|n| n.has_tag_name("SetPreset"))?;
    // let profile_token = set_preset.descendants().find(|n| n.has_tag_name("ProfileToken"))?;
    let preset_name = set_preset
        .descendants()
        .find(|n| n.has_tag_name("PresetName"))?;
    let preset_token = set_preset
        .descendants()
        .find(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("PresetToken"))?;

    return Some((
        preset_name.text().unwrap_or_default().to_string(),
        preset_token.text().unwrap_or_default().to_string(),
    ));
}

fn parse_relative_move_params(xml: &str) -> Option<(String, String, String)> {
    let doc: Document<'_> = Document::parse(xml).ok()?;

    let body = doc.descendants().find(|n| n.has_tag_name("Body"))?;
    let relative_move = body
        .descendants()
        .find(|n| n.has_tag_name("RelativeMove"))?;

    // <Translation><PanTilt x="0.04" y="0" xmlns="http://www.onvif.org/ver10/schema"/><Zoom x="0" xmlns="http://www.onvif.org/ver10/schema"/></Translation><

    let translation = relative_move
        .descendants()
        .find(|n| n.has_tag_name("Translation"))?;

    let pan_tilt = translation
        .descendants()
        .find(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("PanTilt"))?;

    let zoom = translation
        .descendants()
        .find(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("Zoom"))?;

    return Some((
        pan_tilt.attribute("x").unwrap_or_default().to_string(),
        pan_tilt.attribute("y").unwrap_or_default().to_string(),
        zoom.attribute("x").unwrap_or_default().to_string(),
    ));
}

fn parse_go_to_preset_params(xml: &str) -> Option<String> {
    let doc: Document<'_> = Document::parse(xml).ok()?;

    let body = doc.descendants().find(|n| n.has_tag_name("Body"))?;
    let go_to_preset = body.descendants().find(|n| n.has_tag_name("GotoPreset"))?;

    // <ProfileToken>Profile_1</ProfileToken><PresetToken>2</PresetToken><Speed></Speed>

    let preset_token = go_to_preset
        .descendants()
        .find(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("PresetToken"))?;

    return Some(preset_token.text().unwrap_or_default().to_string());
}

fn get_response_for_ptz(action: String, state: &State<SharedCameraState>, xml: &str) -> String {
    if action.ends_with("GetPresets") {
        let s = state.lock().unwrap();
        return build_get_presets_response(&s.presets);
    }
    if action.ends_with("SetPreset") {
        let (preset_name, preset_token) = parse_set_preset_params(&xml).unwrap_or_default();

        let (pan, tilt, zoom) = {
            let s = state.lock().unwrap();
            (s.pan, s.tilt, s.zoom)
        };

        let mut s = state.lock().unwrap();

        if let Some(preset) = s.presets.iter_mut().find(|p| p.token == preset_token) {
            preset.name = preset_name;
            preset.pan = pan;
            preset.tilt = tilt;
            preset.zoom = zoom;
        }
    }
    if action.ends_with("RelativeMove") {
        let (delta_pan, delta_tilt, delta_zoom) =
            parse_relative_move_params(&xml).unwrap_or_default();

        let mut s = state.lock().unwrap();

        s.pan += delta_pan.parse::<f32>().unwrap_or(0.0);
        s.tilt += delta_tilt.parse::<f32>().unwrap_or(0.0);
        s.zoom += delta_zoom.parse::<f32>().unwrap_or(0.0);

        s.pan = s.pan.clamp(-1.0, 1.0);
        s.tilt = s.tilt.clamp(-1.0, 1.0);
        s.zoom = s.zoom.clamp(0.0, 1.0);
    }
    if action.ends_with("GotoPreset") {
        let preset_token = parse_go_to_preset_params(&xml).unwrap_or_default();

        println!("preset_token {}", preset_token);

        let preset_values = {
            let s = state.lock().unwrap();
            s.presets
                .iter()
                .find(|p| p.token == preset_token)
                .map(|p| (p.pan, p.tilt, p.zoom))
        };

        if let Some((pan, tilt, zoom)) = preset_values {
            println!("preset values {} {} {}", pan, tilt, zoom);
            let mut s = state.lock().unwrap();
            s.pan = pan;
            s.tilt = tilt;
            s.zoom = zoom;
        }
    }

    return default_ptz_success_response();
}

fn build_get_presets_response(presets: &[Preset]) -> String {
    let presets_xml = presets
        .iter()
        .map(|preset| {
            format!(
                r#"
<tptz:Preset token="{token}">
  <tt:Name>{name}</tt:Name>
  <tt:PTZPosition>
    <tt:PanTilt x="0.000000" y="0.000000"/>
    <tt:Zoom x="0.000000"/>
  </tt:PTZPosition>
</tptz:Preset>
"#,
                token = preset.token,
                name = preset.name
            )
        })
        .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope>
  <env:Body>
    <tptz:GetPresetsResponse>
      {presets_xml}
    </tptz:GetPresetsResponse>
  </env:Body>
</env:Envelope>"#
    )
}

async fn ptz(state: State<SharedCameraState>, body: Bytes) -> impl IntoResponse {
    let xml = String::from_utf8_lossy(&body);

    let action = detect_onvif_op(&xml).unwrap_or_default();

    eprintln!("action {}", action);

    let response = get_response_for_ptz(action, &state, &xml);

    return (
        StatusCode::OK,
        [("Content-Type", "application/soap+xml; charset=utf-8")],
        response,
    );
}

pub async fn start_http_server(onvif_state: SharedCameraState) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    eprintln!("HTTP server listening on {}", addr);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::GET])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/onvif/device_service", post(device_server))
        .route("/onvif/Media", post(media))
        .route("/onvif/Media2", post(media2))
        .route("/onvif/PTZ", post(ptz))
        .with_state(onvif_state)
        .fallback(fallback_handler)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                info_span!(
                    "http_request",
                    method = ?request.method(),
                    // matched_path,
                    request_uri = request.uri().to_string(),
                    some_other_field = tracing::field::Empty,
                )
            }),
        );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind HTTP server");

    return axum::serve(listener, app)
        .await
        .expect("HTTP server crashed");
}
