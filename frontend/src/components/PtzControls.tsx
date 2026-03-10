import { FaArrowDown, FaArrowLeft, FaArrowRight, FaArrowUp, FaCircle, FaMinus, FaPlay, FaPlus } from "react-icons/fa6";
import { getPresets, gotoPreset, relativeMove, setPreset } from "../services/onvif-client";
import type React from "react";
import { useEffect, useState } from "react";
import { FaSave } from "react-icons/fa";

export default function PTZControls() {
    const [presets, setPresets] = useState<{ token: string | null; name: string | undefined; isDirty?: boolean; }[]>([]);

    useEffect(() => {
        (async () => {
            const presets = await getPresets();
            setPresets(presets);
        })();
    }, []);

    return (
        <div style={styles.ptzControls}>
            <div style={styles.control}>
                <div style={styles.grid}>
                    <div style={styles.buttonRows}>
                        <button onClick={() => relativeMove(-0.1, -0.1, 0)}><FaArrowUp style={{ transform: "rotate(-45deg)" }} /></button>
                        <button onClick={() => relativeMove(0, -0.1, 0)}><FaArrowUp /></button>
                        <button onClick={() => relativeMove(0.1, -0.1, 0)}><FaArrowUp style={{ transform: "rotate(45deg)" }} /></button>
                    </div>
                    <div style={styles.buttonRows}>
                        <button onClick={() => relativeMove(-0.1, 0, 0)}><FaArrowLeft /></button>
                        <button ><FaCircle /></button>
                        <button onClick={() => relativeMove(0.1, 0, 0)}><FaArrowRight /></button>
                    </div>
                    <div style={styles.buttonRows}>
                        <button onClick={() => relativeMove(-0.1, 0.1, 0)}><FaArrowDown style={{ transform: "rotate(45deg)" }} /></button>
                        <button onClick={() => relativeMove(0, 0.1, 0)}><FaArrowDown /></button>
                        <button onClick={() => relativeMove(0.1, 0.1, 0)}><FaArrowDown style={{ transform: "rotate(-45deg)" }} /></button>
                    </div>
                </div>
                <div style={styles.grid}>
                    <div style={styles.buttonRows}>
                        <button onClick={() => relativeMove(0, 0, 0.1)}><FaPlus /></button>
                    </div>
                    <div style={styles.buttonRows}>
                        <button onClick={() => relativeMove(0, 0, -0.1)}><FaMinus /></button>
                    </div>
                </div>
            </div>
            <div style={styles.presets}>
                {presets.map((preset) => <div style={styles.preset}>
                    <button onClick={() => preset.token && gotoPreset(preset.token)}><FaPlay /></button>
                    <input value={preset.name} onChange={(e) => {
                        const newName = e.target.value;
                        setPresets(prev =>
                            prev.map(p =>
                                p.token === preset.token ? { ...p, name: newName, isDirty: true } : p
                            )
                        );
                    }} />
                    <div style={{ width: 50 }}>
                        {preset.isDirty &&
                            <button onClick={() => {
                                if (!preset.token || !preset.name) {
                                    return;
                                }
                                setPreset(preset.token, preset.name);
                                setPresets(prev =>
                                    prev.map(p =>
                                        p.token === preset.token ? { ...p, isDirty: false } : p
                                    )
                                );
                            }}><FaSave /></button>
                        }
                    </div>

                </div>)}
            </div>
        </div >
    );
}

const styles: { [component: string]: React.CSSProperties; } = {
    preset: {
        display: 'flex',
        flex: 1,
        flexDirection: 'row',
        gap: 6,
    },
    grid: {
        display: 'flex',
        flex: 1,
        flexDirection: 'column',
        gap: 3,
        justifyContent: 'center',
    },
    buttonRows: {
        display: 'flex',
        flex: 1,
        flexDirection: 'row',
        maxHeight: 50,
        gap: 3,
    },
    control: {
        display: 'flex',
        flex: 1,
        flexDirection: 'row',
        gap: 20,
        maxHeight: 200,
    },
    ptzControls: {
        display: 'flex',
        flex: 1,
        gap: 20,
        flexDirection: 'column',
        maxWidth: 500,
    },
    presets: {
        display: 'flex',
        flex: 1,
        flexDirection: 'column',
        gap: 3,
        maxHeight: 600,
    }
};