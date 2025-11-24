import init, {
    init_reader,
    init_scanner,
    read_from_image,
    start_stream_scan,
    stop_stream_scan,
    on_detect,
    on_start,
    on_stop,
} from "../pkg/wascan.js"

init()
    .then(() => {
        init_reader()
        init_scanner()

        on_start(() => {
            console.log("started")
        })

        on_detect((result) => {
            if (result.success) {
                console.log(result.value)
                alert(result.value)
            } else {
                console.error(result.error)
            }
        })

        on_stop(() => {
            console.log("stopped")
        })

        document.getElementById("read-from-image-button").addEventListener("click", () => {
            read_from_image()
        })

        document.getElementById("start-stream-scan-button").addEventListener("click", () => {
            start_stream_scan("stream-video-element")
        })

        document.getElementById("stop-stream-scan-button").addEventListener("click", () => {
            stop_stream_scan()
        })
    })
    .catch(e => console.error(e))
