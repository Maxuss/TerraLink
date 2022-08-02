package space.maxus.terralink.net;

import space.maxus.terralink.TerraLink;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;

public class BufferUtil {
    private static final int MAX_STR_LEN = Short.MAX_VALUE;

    public static void writeStr(BufferedOutputStream to, String str) throws IOException {
        var bytes = str.getBytes(StandardCharsets.UTF_8);
        if(bytes.length > MAX_STR_LEN) {
            TerraLink.LOGGER.warn("Writing string bytes of over MAX_STR_LEN (" + bytes.length + " > " + MAX_STR_LEN + "), this might disconnect client from older versions of the bridge.");
        }
        var buf = ByteBuffer.allocate(4);
        buf.putInt(bytes.length);
        to.write(buf.array());
        to.write(bytes);
    }

    @SuppressWarnings("ResultOfMethodCallIgnored")
    public static String readStr(BufferedInputStream from) throws IOException {
        var intBytes = new byte[4];
        from.read(intBytes);
        var len = ByteBuffer.wrap(intBytes).getInt();
        var strBuf = new byte[len];
        from.read(strBuf);
        return StandardCharsets.UTF_8.decode(ByteBuffer.wrap(strBuf)).toString();
    }
}
