import { useEffect, useState } from 'react';
import './App.css';
import CameraPlayer from './components/CameraPlayer';
import PTZControls from './components/PtzControls';

function App() {

  const [hls, setHls] = useState('');
  const [currentPage, setCurrentPage] = useState<'liveView' | 'configuration'>('liveView');


  useEffect(() => {
    (async () => {
      const hlsUrl = new URL(window.location.origin);
      hlsUrl.port = '8888';
      setHls(`${hlsUrl.origin}/cam1/index.m3u8`);
    })();
  }, []);

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 120, borderColor: '#333333', borderWidth: 1, borderStyle: 'solid' }}>
      <div style={{ maxHeight: 50, display: 'flex', flex: 1, flexDirection: 'row', backgroundColor: '#333333', padding: 5, gap: 5 }}>
        <button
          onClick={() => setCurrentPage('liveView')}
          style={currentPage === 'liveView' ? { backgroundColor: '#ff3333' } : {}}
          disabled={currentPage === 'liveView'}
        >Live View</button>
        <button
          onClick={() => setCurrentPage('configuration')}
          style={currentPage === 'configuration' ? { backgroundColor: '#ff3333' } : {}}
          disabled={currentPage === 'configuration'}
        >Configuration</button>
      </div>
      <div style={{
        flex: 1,
        flexDirection: 'row',
        display: 'flex',
        gap: 60,
      }}>
        {currentPage === 'liveView' &&
          <>
            {hls &&
              <CameraPlayer src="http://localhost:8888/cam1/index.m3u8" />
            }
            <PTZControls />
          </>
        }
      </div>
    </div>
  );
}

export default App;
