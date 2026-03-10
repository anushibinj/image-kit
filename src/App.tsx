import { useState, useEffect } from 'react'

function App() {
  const [text, setText] = useState("Image Kit ")
  const [fontSize, setFontSize] = useState(32)
  const [textColor, setTextColor] = useState("#000000")
  const [bgColor, setBgColor] = useState("#ffffff")
  const [width, setWidth] = useState(800)
  const [height, setHeight] = useState(600)
  const [horizontalSpacing, setHorizontalSpacing] = useState(20)
  const [verticalSpacing, setVerticalSpacing] = useState(20)
  const [rotationAngle, setRotationAngle] = useState(15)

  const [imageSrc, setImageSrc] = useState<string | null>(null)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [wasmModule, setWasmModule] = useState<any>(null)

  useEffect(() => {
    let active = true;
    import('./wasm/wasm_core.js').then(async (module) => {
      await module.default()
      if (active) {
        setWasmModule(module)
      }
    }).catch(console.error)

    return () => { active = false }
  }, [])

  useEffect(() => {
    if (!wasmModule) return

    try {
      const buffer = wasmModule.generate_text_pattern(
        text,
        fontSize,
        textColor,
        bgColor,
        width,
        height,
        horizontalSpacing,
        verticalSpacing,
        rotationAngle
      )

      const blob = new Blob([buffer], { type: 'image/png' })
      const url = URL.createObjectURL(blob)
      setImageSrc(url)

      return () => {
        URL.revokeObjectURL(url)
      }
    } catch (e) {
      console.error(e)
    }
  }, [text, fontSize, textColor, bgColor, width, height, horizontalSpacing, verticalSpacing, rotationAngle, wasmModule])

  return (
    <div className="min-h-screen bg-gray-100 flex p-6 gap-6 font-sans">
      <div className="w-80 flex-shrink-0 bg-white p-6 rounded-lg shadow-sm overflow-y-auto">
        <h1 className="text-2xl font-bold mb-6 text-gray-800">Image Kit</h1>

        <div className="space-y-6">
          <section>
            <h2 className="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">Text Settings</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Text</label>
                <input
                  type="text"
                  value={text}
                  onChange={(e) => setText(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Font Size: {fontSize}px</label>
                <input
                  type="range"
                  min="8" max="120"
                  value={fontSize}
                  onChange={(e) => setFontSize(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div className="flex gap-4">
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
                  <input
                    type="color"
                    value={textColor}
                    onChange={(e) => setTextColor(e.target.value)}
                    className="w-full h-10 p-1 border border-gray-300 rounded-md cursor-pointer"
                  />
                </div>
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Background</label>
                  <input
                    type="color"
                    value={bgColor}
                    onChange={(e) => setBgColor(e.target.value)}
                    className="w-full h-10 p-1 border border-gray-300 rounded-md cursor-pointer"
                  />
                </div>
              </div>
            </div>
          </section>

          <section>
            <h2 className="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">Canvas Settings</h2>

            <div className="space-y-4">
              <div className="flex gap-4">
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Width</label>
                  <input
                    type="number"
                    value={width}
                    onChange={(e) => setWidth(Number(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Height</label>
                  <input
                    type="number"
                    value={height}
                    onChange={(e) => setHeight(Number(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Rotation Angle: {rotationAngle}°</label>
                <input
                  type="range"
                  min="-180" max="180"
                  value={rotationAngle}
                  onChange={(e) => setRotationAngle(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Horizontal Spacing: {horizontalSpacing}px</label>
                <input
                  type="range"
                  min="0" max="100"
                  value={horizontalSpacing}
                  onChange={(e) => setHorizontalSpacing(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Vertical Spacing: {verticalSpacing}px</label>
                <input
                  type="range"
                  min="0" max="100"
                  value={verticalSpacing}
                  onChange={(e) => setVerticalSpacing(Number(e.target.value))}
                  className="w-full"
                />
              </div>
            </div>
          </section>
        </div>
      </div>

      <div className="flex-1 bg-white rounded-lg shadow-sm p-6 flex flex-col">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold text-gray-800">Preview</h2>
          <div id="export-actions">
            <button
              onClick={() => {
                if (imageSrc) {
                  const link = document.createElement('a');
                  link.href = imageSrc;
                  link.download = 'pattern.png';
                  document.body.appendChild(link);
                  link.click();
                  document.body.removeChild(link);
                }
              }}
              disabled={!imageSrc}
              className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Download Image (PNG)
            </button>
          </div>
        </div>

        <div className="flex-1 bg-gray-50 rounded border-2 border-dashed border-gray-200 flex items-center justify-center overflow-auto p-4">
          {imageSrc ? (
            <img
              src={imageSrc}
              alt="Generated Pattern"
              className="max-w-full shadow-md bg-white"
              style={{ maxHeight: '100%', objectFit: 'contain' }}
            />
          ) : (
            <div className="text-gray-400">Loading generator...</div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
