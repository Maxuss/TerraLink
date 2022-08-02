package space.maxus.terralink.net;

import java.io.BufferedInputStream;
import java.io.IOException;

public interface PacketReader<P> {
    P readPacket(BufferedInputStream is) throws IOException;

    static PacketReader<Packet> nullReader() {
        return is -> null;
    }
}
