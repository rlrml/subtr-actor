import React, { useState } from 'react';
import { Settings, X } from 'lucide-react';
import { EnvironmentSelector } from './EnvironmentSelector';

const STORAGE_KEY = 'rl-viewer-settings';

const defaultSettings = {
    customEnvironmentId: null, // Custom environment from API (null = default scene)
    // Display settings
    showHitboxes: false,
    showBallSpeed: false,
    showCarSpeed: false,
    speedUnit: 'kmh', // 'kmh' or 'mph'
};

export function useSettings() {
    const [settings, setSettings] = useState(() => {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            if (stored) {
                return { ...defaultSettings, ...JSON.parse(stored) };
            }
        } catch (e) {
            console.warn('[Settings] Failed to load from localStorage:', e);
        }
        return defaultSettings;
    });

    const updateSettings = (newSettings) => {
        const updated = { ...settings, ...newSettings };
        setSettings(updated);
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
        } catch (e) {
            console.warn('[Settings] Failed to save to localStorage:', e);
        }
    };

    return [settings, updateSettings];
}

export function SettingsPanel({ settings, onSettingsChange }) {
    const [isOpen, setIsOpen] = useState(false);
    const [activeTab, setActiveTab] = useState('graphics');

    const tabs = [
        { id: 'graphics', label: 'Graphics' },
        { id: 'display', label: 'Display' },
        { id: 'debug', label: 'Debug' },
    ];

    return (
        <>
            {/* Settings Button */}
            <button
                onClick={() => setIsOpen(true)}
                className="pointer-events-auto bg-black/50 hover:bg-black/70 p-2 rounded backdrop-blur-sm transition-colors"
                title="Settings"
            >
                <Settings size={20} className="text-white" />
            </button>

            {/* Settings Modal */}
            {isOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-auto">
                    {/* Backdrop */}
                    <div
                        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                        onClick={() => setIsOpen(false)}
                    />

                    {/* Modal */}
                    <div className="relative bg-gray-900 border border-gray-700 rounded-lg shadow-2xl w-[500px] max-h-[85vh] flex flex-col">
                        {/* Header */}
                        <div className="flex items-center justify-between p-4 border-b border-gray-700 flex-shrink-0">
                            <div className="flex items-center gap-2 text-white">
                                <Settings size={20} />
                                <span className="font-bold text-lg">Settings</span>
                            </div>
                            <button
                                onClick={() => setIsOpen(false)}
                                className="text-gray-400 hover:text-white transition-colors"
                            >
                                <X size={20} />
                            </button>
                        </div>

                        {/* Tabs */}
                        <div className="flex border-b border-gray-700 px-4 flex-shrink-0">
                            {tabs.map((tab) => (
                                <button
                                    key={tab.id}
                                    onClick={() => setActiveTab(tab.id)}
                                    className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 ${
                                        activeTab === tab.id
                                            ? 'border-blue-500 text-white'
                                            : 'border-transparent text-gray-400 hover:text-gray-300'
                                    }`}
                                >
                                    {tab.label}
                                </button>
                            ))}
                        </div>

                        {/* Content */}
                        <div className="p-4 space-y-6 overflow-y-auto flex-1">
                            {/* Graphics Tab */}
                            {activeTab === 'graphics' && (
                                <>
                            {/* Environment Section */}
                            <div className="space-y-4">
                                <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                                    Environment
                                </h3>

                                {/* Custom Environment Selector */}
                                <EnvironmentSelector
                                    currentEnvironmentId={settings.customEnvironmentId}
                                    onEnvironmentChange={(envId) => {
                                        onSettingsChange({ customEnvironmentId: envId || null });
                                    }}
                                />
                                <p className="text-xs text-gray-500">
                                    Select a custom environment with meshes, lights, and skybox settings
                                </p>
                            </div>
                                </>
                            )}

                            {/* Display Tab */}
                            {activeTab === 'display' && (
                                <>
                            {/* Speed Display Section */}
                            <div className="space-y-4">
                                <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                                    Speed Display
                                </h3>

                                {/* Show Ball Speed Toggle */}
                                <div className="flex items-center justify-between">
                                    <div>
                                        <label className="text-sm text-gray-300">Show Ball Speed</label>
                                        <p className="text-xs text-gray-500">
                                            Display speed label near the ball
                                        </p>
                                    </div>
                                    <button
                                        onClick={() => onSettingsChange({ showBallSpeed: !settings.showBallSpeed })}
                                        className={`relative w-11 h-6 rounded-full transition-colors ${
                                            settings.showBallSpeed ? 'bg-blue-600' : 'bg-gray-600'
                                        }`}
                                    >
                                        <span
                                            className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                                                settings.showBallSpeed ? 'translate-x-5' : 'translate-x-0'
                                            }`}
                                        />
                                    </button>
                                </div>

                                {/* Show Car Speed Toggle */}
                                <div className="flex items-center justify-between">
                                    <div>
                                        <label className="text-sm text-gray-300">Show Car Speed</label>
                                        <p className="text-xs text-gray-500">
                                            Display speed labels near cars
                                        </p>
                                    </div>
                                    <button
                                        onClick={() => onSettingsChange({ showCarSpeed: !settings.showCarSpeed })}
                                        className={`relative w-11 h-6 rounded-full transition-colors ${
                                            settings.showCarSpeed ? 'bg-blue-600' : 'bg-gray-600'
                                        }`}
                                    >
                                        <span
                                            className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                                                settings.showCarSpeed ? 'translate-x-5' : 'translate-x-0'
                                            }`}
                                        />
                                    </button>
                                </div>

                                {/* Speed Unit Selector */}
                                <div className="space-y-2">
                                    <label className="text-sm text-gray-300">Speed Unit</label>
                                    <div className="flex gap-2">
                                        <button
                                            onClick={() => onSettingsChange({ speedUnit: 'kmh' })}
                                            className={`flex-1 py-2 px-4 rounded text-sm font-medium transition-colors ${
                                                settings.speedUnit === 'kmh'
                                                    ? 'bg-blue-600 text-white'
                                                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                                            }`}
                                        >
                                            km/h
                                        </button>
                                        <button
                                            onClick={() => onSettingsChange({ speedUnit: 'mph' })}
                                            className={`flex-1 py-2 px-4 rounded text-sm font-medium transition-colors ${
                                                settings.speedUnit === 'mph'
                                                    ? 'bg-blue-600 text-white'
                                                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                                            }`}
                                        >
                                            mph
                                        </button>
                                    </div>
                                </div>
                            </div>
                                </>
                            )}

                            {/* Debug Tab */}
                            {activeTab === 'debug' && (
                                <>
                            {/* Debug Section */}
                            <div className="space-y-4">
                                <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                                    Debug
                                </h3>

                                {/* Show Hitboxes Toggle */}
                                <div className="flex items-center justify-between">
                                    <div>
                                        <label className="text-sm text-gray-300">Show Car Hitboxes</label>
                                        <p className="text-xs text-gray-500">
                                            Display wireframe hitbox overlays on cars
                                        </p>
                                    </div>
                                    <button
                                        onClick={() => onSettingsChange({ showHitboxes: !settings.showHitboxes })}
                                        className={`relative w-11 h-6 rounded-full transition-colors ${
                                            settings.showHitboxes ? 'bg-blue-600' : 'bg-gray-600'
                                        }`}
                                    >
                                        <span
                                            className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                                                settings.showHitboxes ? 'translate-x-5' : 'translate-x-0'
                                            }`}
                                        />
                                    </button>
                                </div>
                            </div>
                                </>
                            )}
                        </div>

                        {/* Footer */}
                        <div className="p-4 border-t border-gray-700 flex-shrink-0">
                            <button
                                onClick={() => {
                                    onSettingsChange(defaultSettings);
                                }}
                                className="text-sm text-gray-400 hover:text-white transition-colors"
                            >
                                Reset to defaults
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
