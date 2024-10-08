import Foundation
//import ProtoBridge
import ProtoBridge
import SwiftProtobuf

func receiveMsg(_ process: @escaping (Ffi_Messaging_FromRust) -> Void) -> (
    (UnsafePointer<UInt8>?, Int, UnsafeMutableRawPointer?) -> Void
) {
    let retfunc: (UnsafePointer<UInt8>?, Int, UnsafeMutableRawPointer?) -> Void = {
        data, len, userData in
        guard let data = data else { return }
        let responseData = Data(bytes: data, count: len)
        do {
            let responseProto = try Ffi_Messaging_FromRust(serializedBytes: responseData)
            // Process the response
            process(responseProto)
            print("Received response: \(responseProto)")
        } catch {
            print("Failed to parse response: \(error)")
        }
    }
    return retfunc
}
// Callback function
public func protoCallback(data: UnsafePointer<UInt8>?, len: Int, userData: UnsafeMutableRawPointer?)
{
    guard let data = data else { return }
    let responseData = Data(bytes: data, count: len)
    do {
        let responseProto = try Ffi_Messaging_FromRust(serializedBytes: responseData)
        // Process the response
        print("Received response: \(responseProto)")
    } catch {
        print("Failed to parse response: \(error)")
    }
}

enum InteropError: Error {
    case serializeRequestFailure
}
public typealias FromRust = Ffi_Messaging_FromRust
public typealias ToRust = Ffi_Messaging_ToRust
// Function to send request
public func sendRequest(
    _ cf: callback_fn = protoCallback,
    request: ToRust = ToRust.with { message in
        message.requestAdd = Ffi_Messaging_add.with {
            $0.v1 = 1
            $0.v2 = 1
        }
    }
) throws {  // * Create and populate your request
    // *

    do {
        let requestData = try request.serializedData()
        requestData.withUnsafeBytes { (bufferPointer: UnsafeRawBufferPointer) in
            let rawPtr = bufferPointer.baseAddress!
            process_proto(
                rawPtr.assumingMemoryBound(to: UInt8.self),
                requestData.count,
                cf,
                nil)
        }
    } catch {
        throw InteropError.serializeRequestFailure
    }
}
