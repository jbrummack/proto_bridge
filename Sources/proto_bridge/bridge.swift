import Foundation

// Callback function
func protoCallback(data: UnsafePointer<UInt8>?, len: Int, userData: UnsafeMutableRawPointer?) {
    guard let data = data else { return }
    let responseData = Data(bytes: data, count: len)
    do {
        let responseProto = try Ffi_Messaging_FromRust(serializedData: responseData)
        // Process the response
        print("Received response: \(responseProto)")
    } catch {
        print("Failed to parse response: \(error)")
    }
}

// Function to send request
func sendRequest() {
    let request = Ffi_Messaging_ToRust()  // Create and populate your request
    do {
        let requestData = try request.serializedData()
        requestData.withUnsafeBytes { (bufferPointer: UnsafeRawBufferPointer) in
            let rawPtr = bufferPointer.baseAddress!
            process_proto(
                rawPtr.assumingMemoryBound(to: UInt8.self),
                requestData.count,
                protoCallback,
                nil)
        }
    } catch {
        print("Failed to serialize request: \(error)")
    }
}
