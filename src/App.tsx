import { useState, useEffect } from 'react'

function App() {
  const [text, setText] = useState("Image Kit ")
  const [fontFamily, setFontFamily] = useState("OpenSans")
  const [fontSize, setFontSize] = useState<number | string>(32)
  const [textColor, setTextColor] = useState("#000000")
  const [bgColor, setBgColor] = useState("#ffffff")
  const [width, setWidth] = useState<number | string>(800)
  const [height, setHeight] = useState<number | string>(600)
  const [horizontalSpacing, setHorizontalSpacing] = useState<number | string>(20)
  const [verticalSpacing, setVerticalSpacing] = useState<number | string>(20)
  const [rotationAngle, setRotationAngle] = useState<number | string>(15)

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

    const parsedFontSize = Number(fontSize) || 32;
    const parsedWidth = Number(width) || 800;
    const parsedHeight = Number(height) || 600;
    const parsedHSpacing = Number(horizontalSpacing) || 0;
    const parsedVSpacing = Number(verticalSpacing) || 0;
    const parsedRotation = Number(rotationAngle) || 0;

    try {
      const buffer = wasmModule.generate_text_pattern(
        text,
        fontFamily,
        parsedFontSize,
        textColor,
        bgColor,
        parsedWidth,
        parsedHeight,
        parsedHSpacing,
        parsedVSpacing,
        parsedRotation
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
  }, [text, fontFamily, fontSize, textColor, bgColor, width, height, horizontalSpacing, verticalSpacing, rotationAngle, wasmModule])

  const fonts = ["OpenSans", "Roboto", "Lora", "Pacifico"];

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
                <label className="block text-sm font-medium text-gray-700 mb-1">Font Family</label>
                <select
                  value={fontFamily}
                  onChange={(e) => setFontFamily(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
                >
                  {fonts.map(font => (
                    <option key={font} value={font}>{font}</option>
                  ))}
                </select>
              </div>

              <div>
                <div className="flex justify-between items-center mb-1">
                  <label className="block text-sm font-medium text-gray-700">Font Size</label>
                  <input
                    type="number"
                    value={fontSize}
                    onChange={(e) => setFontSize(e.target.value)}
                    className="w-16 px-1 py-0.5 text-right text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <input
                  type="range"
                  min="8" max="120"
                  value={Number(fontSize) || 32}
                  onChange={(e) => setFontSize(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div className="flex gap-4">
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
                  <input
                    type="color"
                    value={textColor !== "transparent" ? textColor.slice(0, 7) : "#000000"}
                    onChange={(e) => setTextColor(e.target.value)}
                    className="w-full h-10 p-1 border border-gray-300 rounded-md cursor-pointer"
                  />
                </div>
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Background</label>
                  <input
                    type="color"
                    value={bgColor !== "transparent" ? bgColor.slice(0, 7) : "#ffffff"}
                    onChange={(e) => setBgColor(e.target.value)}
                    className="w-full h-10 p-1 border border-gray-300 rounded-md cursor-pointer"
                  />
                  <div className="mt-1 flex items-center">
                    <input
                      type="checkbox"
                      id="bg-transparent"
                      checked={bgColor === "transparent"}
                      onChange={(e) => setBgColor(e.target.checked ? "transparent" : "#ffffff")}
                      className="mr-2"
                    />
                    <label htmlFor="bg-transparent" className="text-xs text-gray-600">Transparent</label>
                  </div>
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
                    onChange={(e) => setWidth(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div className="flex-1">
                  <label className="block text-sm font-medium text-gray-700 mb-1">Height</label>
                  <input
                    type="number"
                    value={height}
                    onChange={(e) => setHeight(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div>
                <div className="flex justify-between items-center mb-1">
                  <label className="block text-sm font-medium text-gray-700">Rotation Angle</label>
                  <input
                    type="number"
                    value={rotationAngle}
                    onChange={(e) => setRotationAngle(e.target.value)}
                    className="w-16 px-1 py-0.5 text-right text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <input
                  type="range"
                  min="-180" max="180"
                  value={Number(rotationAngle) || 0}
                  onChange={(e) => setRotationAngle(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between items-center mb-1">
                  <label className="block text-sm font-medium text-gray-700">Horizontal Spacing</label>
                  <input
                    type="number"
                    value={horizontalSpacing}
                    onChange={(e) => setHorizontalSpacing(e.target.value)}
                    className="w-16 px-1 py-0.5 text-right text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <input
                  type="range"
                  min="0" max="100"
                  value={Number(horizontalSpacing) || 0}
                  onChange={(e) => setHorizontalSpacing(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between items-center mb-1">
                  <label className="block text-sm font-medium text-gray-700">Vertical Spacing</label>
                  <input
                    type="number"
                    value={verticalSpacing}
                    onChange={(e) => setVerticalSpacing(e.target.value)}
                    className="w-16 px-1 py-0.5 text-right text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <input
                  type="range"
                  min="0" max="100"
                  value={Number(verticalSpacing) || 0}
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

        <div className="flex-1 bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyMCIgaGVpZ2h0PSIyMCI+CjxyZWN0IHdpZHRoPSIyMCIgaGVpZ2h0PSIyMCIgZmlsbD0iI2ZmZiIgLz4KPHJlY3QgeD0iMCIgeT0iMCIgd2lkdGg9IjEwIiBoZWlnaHQ9IjEwIiBmaWxsPSIjZjBmMGYwIiAvPgo8cmVjdCB4PSIxMCIgeT0iMTAiIHdpZHRoPSIxMCIgaGVpZ2h0PSIxMCIgZmlsbD0iI2YwZjBmMCIgLz4KPC9zdmc+')] rounded border-2 border-dashed border-gray-200 flex items-center justify-center overflow-auto p-4">
          {imageSrc ? (
            <img
              src={imageSrc}
              alt="Generated Pattern"
              className="max-w-full shadow-md"
              style={{ maxHeight: '100%', objectFit: 'contain' }}
            />
          ) : (
            <div className="text-gray-400 bg-white p-4 rounded">Loading generator...</div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
