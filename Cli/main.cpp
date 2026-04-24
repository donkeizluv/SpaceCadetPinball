#include <algorithm>
#include <cerrno>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

#ifdef _WIN32
#include <direct.h>
#include <sys/stat.h>
#else
#include <sys/stat.h>
#include <sys/types.h>
#endif

enum class FieldTypes : int16_t
{
    ShortValue = 0,
    Bitmap8bit = 1,
    Unknown2 = 2,
    GroupName = 3,
    Unknown4 = 4,
    Palette = 5,
    Unknown6 = 6,
    Unknown7 = 7,
    Unknown8 = 8,
    String = 9,
    ShortArray = 10,
    FloatArray = 11,
    Bitmap16bit = 12,
};

enum class bmp8Flags : unsigned char
{
    RawBmpUnaligned = 1 << 0,
    DibBitmap = 1 << 1,
    Spliced = 1 << 2,
};

#pragma pack(push, 1)
struct datFileHeader
{
    char FileSignature[21];
    char AppName[50];
    char Description[100];
    int32_t FileSize;
    uint16_t NumberOfGroups;
    int32_t SizeOfBody;
    uint16_t Unknown;
};

struct dat8BitBmpHeader
{
    uint8_t Resolution;
    int16_t Width;
    int16_t Height;
    int16_t XPosition;
    int16_t YPosition;
    int32_t Size;
    bmp8Flags Flags;

    bool IsFlagSet(bmp8Flags flag) const
    {
        return (static_cast<unsigned char>(Flags) & static_cast<unsigned char>(flag)) != 0;
    }
};

struct dat16BitBmpHeader
{
    int16_t Width;
    int16_t Height;
    int16_t Stride;
    int32_t Unknown0;
    int16_t Unknown1_0;
    int16_t Unknown1_1;
};

struct bmpFileHeader
{
    uint16_t Signature;
    uint32_t FileSize;
    uint16_t Reserved1;
    uint16_t Reserved2;
    uint32_t PixelDataOffset;
};

struct bmpInfoHeader
{
    uint32_t HeaderSize;
    int32_t Width;
    int32_t Height;
    uint16_t Planes;
    uint16_t BitCount;
    uint32_t Compression;
    uint32_t SizeImage;
    int32_t XPelsPerMeter;
    int32_t YPelsPerMeter;
    uint32_t ClrUsed;
    uint32_t ClrImportant;
};
#pragma pack(pop)

static const int16_t kFieldSize[] = {2, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0};

static bool ReadBytes(FILE *file, void *out, size_t size)
{
    return std::fread(out, 1, size, file) == size;
}

template <typename T>
static bool ReadValue(FILE *file, T &out)
{
    return ReadBytes(file, &out, sizeof(T));
}

static bool WriteBinaryFile(const std::string &path, const void *data, size_t size)
{
    std::ofstream out(path.c_str(), std::ios::binary);
    if (!out)
        return false;

    out.write(reinterpret_cast<const char *>(data), static_cast<std::streamsize>(size));
    return out.good();
}

static std::vector<uint8_t> BuildBmpPalette(const std::vector<uint8_t> &paletteData)
{
    std::vector<uint8_t> palette(256u * 4u, 0);

    auto setPaletteColor = [&](size_t index, uint8_t red, uint8_t green, uint8_t blue)
    {
        palette[index * 4 + 0] = blue;
        palette[index * 4 + 1] = green;
        palette[index * 4 + 2] = red;
        palette[index * 4 + 3] = 0;
    };

    // Keep these entries aligned with the game default palette bootstrap.
    setPaletteColor(0, 0, 0, 0);
    setPaletteColor(1, 0x80, 0, 0);
    setPaletteColor(2, 0, 0x80, 0);
    setPaletteColor(3, 0x80, 0x80, 0);
    setPaletteColor(4, 0, 0, 0x80);
    setPaletteColor(5, 0x80, 0, 0x80);
    setPaletteColor(6, 0, 0x80, 0x80);
    setPaletteColor(7, 0xC0, 0xC0, 0xC0);
    setPaletteColor(8, 0xC0, 0xDC, 0xC0);
    setPaletteColor(9, 0xA6, 0xCA, 0xF0);
    setPaletteColor(255, 0xFF, 0xFF, 0xFF);

    if (paletteData.size() >= 246u * 4u)
    {
        for (size_t i = 10; i < 246; ++i)
        {
            const size_t src = i * 4;
            palette[src + 0] = paletteData[src + 0];
            palette[src + 1] = paletteData[src + 1];
            palette[src + 2] = paletteData[src + 2];
            palette[src + 3] = 0;
        }
    }

    return palette;
}

static bool WriteIndexedBmp(const std::string &path, int width, int height, const std::vector<char> &payload,
                            const std::vector<uint8_t> &paletteData)
{
    if (width <= 0 || height <= 0)
        return false;

    const uint32_t srcStride = static_cast<uint32_t>((width + 3) & ~3);
    const uint32_t dstStride = srcStride;
    const uint32_t pixelDataSize = dstStride * static_cast<uint32_t>(height);
    if (payload.size() < pixelDataSize)
        return false;

    const uint32_t paletteSize = 256u * 4u;
    const uint32_t pixelOffset = static_cast<uint32_t>(sizeof(bmpFileHeader) + sizeof(bmpInfoHeader)) + paletteSize;
    const uint32_t fileSize = pixelOffset + pixelDataSize;

    bmpFileHeader fileHeader{};
    fileHeader.Signature = 0x4D42; // 'BM'
    fileHeader.FileSize = fileSize;
    fileHeader.PixelDataOffset = pixelOffset;

    bmpInfoHeader infoHeader{};
    infoHeader.HeaderSize = sizeof(bmpInfoHeader);
    infoHeader.Width = width;
    infoHeader.Height = height;
    infoHeader.Planes = 1;
    infoHeader.BitCount = 8;
    infoHeader.Compression = 0;
    infoHeader.SizeImage = pixelDataSize;
    infoHeader.XPelsPerMeter = 2835;
    infoHeader.YPelsPerMeter = 2835;
    infoHeader.ClrUsed = 256;
    infoHeader.ClrImportant = 256;

    auto palette = BuildBmpPalette(paletteData);

    std::vector<uint8_t> out(fileSize, 0);
    std::memcpy(out.data(), &fileHeader, sizeof(fileHeader));
    std::memcpy(out.data() + sizeof(fileHeader), &infoHeader, sizeof(infoHeader));
    std::memcpy(out.data() + sizeof(fileHeader) + sizeof(infoHeader), palette.data(), paletteSize);

    // BMP stores rows bottom-up when height is positive.
    for (int y = 0; y < height; ++y)
    {
        const size_t srcRow = static_cast<size_t>(height - 1 - y) * srcStride;
        const size_t dstRow = pixelOffset + static_cast<size_t>(y) * dstStride;
        std::memcpy(out.data() + dstRow, payload.data() + srcRow, dstStride);
    }

    return WriteBinaryFile(path, out.data(), out.size());
}

static std::string CleanCString(const std::vector<char> &data)
{
    auto endIt = std::find(data.begin(), data.end(), '\0');
    return std::string(data.begin(), endIt);
}

static std::string CleanFixedString(const char *buf, size_t size)
{
    const char *end = static_cast<const char *>(std::memchr(buf, '\0', size));
    if (end)
        return std::string(buf, end);
    return std::string(buf, buf + size);
}

static std::string FieldTypeName(FieldTypes type)
{
    switch (type)
    {
    case FieldTypes::ShortValue:
        return "short_value";
    case FieldTypes::Bitmap8bit:
        return "bitmap8";
    case FieldTypes::Unknown2:
        return "unknown2";
    case FieldTypes::GroupName:
        return "group_name";
    case FieldTypes::Unknown4:
        return "unknown4";
    case FieldTypes::Palette:
        return "palette";
    case FieldTypes::Unknown6:
        return "unknown6";
    case FieldTypes::Unknown7:
        return "unknown7";
    case FieldTypes::Unknown8:
        return "unknown8";
    case FieldTypes::String:
        return "string";
    case FieldTypes::ShortArray:
        return "short_array";
    case FieldTypes::FloatArray:
        return "float_array";
    case FieldTypes::Bitmap16bit:
        return "bitmap16";
    default:
        return "unknown";
    }
}

static bool DirectoryExists(const std::string &path)
{
#ifdef _WIN32
    struct _stat info{};
    return ::_stat(path.c_str(), &info) == 0 && (info.st_mode & _S_IFDIR) != 0;
#else
    struct stat info;
    return stat(path.c_str(), &info) == 0 && S_ISDIR(info.st_mode);
#endif
}

static bool CreateDirectorySingle(const std::string &path)
{
#ifdef _WIN32
    if (_mkdir(path.c_str()) == 0)
        return true;
#else
    if (mkdir(path.c_str(), 0755) == 0)
        return true;
#endif
    return errno == EEXIST && DirectoryExists(path);
}

static bool EnsureDirectory(const std::string &path)
{
    if (path.empty())
        return false;

    std::string normalized = path;
    std::replace(normalized.begin(), normalized.end(), '\\', '/');

    if (DirectoryExists(normalized))
        return true;

    std::string cur;
    if (normalized.size() >= 2 && normalized[1] == ':')
    {
        cur = normalized.substr(0, 2);
    }
    if (!normalized.empty() && (normalized[0] == '/'))
    {
        cur = "/";
    }

    std::stringstream ss(normalized);
    std::string part;
    while (std::getline(ss, part, '/'))
    {
        if (part.empty())
            continue;

        if (cur.empty() || cur == "/")
            cur += part;
        else if (cur.size() == 2 && cur[1] == ':')
            cur += "/" + part;
        else
            cur += "/" + part;

        if (!DirectoryExists(cur) && !CreateDirectorySingle(cur))
            return false;
    }

    return DirectoryExists(normalized);
}

static std::string JoinPath(const std::string &a, const std::string &b)
{
    if (a.empty())
        return b;
    if (b.empty())
        return a;

    if (a[a.size() - 1] == '/' || a[a.size() - 1] == '\\')
        return a + b;

    return a + "/" + b;
}

static void PrintUsage()
{
    std::cout << "SpaceCadetPinball CLI\n"
              << "Usage:\n"
              << "  SpaceCadetPinballCli extract <input.dat> [output_dir] [--full-tilt] [--bmp]\n";
}

static int ExtractDat(const std::string &inputPath, const std::string &outputDir, bool fullTiltMode, bool exportBmp)
{
    FILE *file = std::fopen(inputPath.c_str(), "rb");
    if (!file)
    {
        std::cerr << "Failed to open input DAT: " << inputPath << "\n";
        return 1;
    }

    datFileHeader header{};
    if (!ReadValue(file, header))
    {
        std::cerr << "Failed to read DAT header\n";
        std::fclose(file);
        return 1;
    }

    if (std::strncmp(header.FileSignature, "PARTOUT(4.0)RESOURCE", 21) != 0)
    {
        std::cerr << "Invalid DAT signature\n";
        std::fclose(file);
        return 1;
    }

    if (!EnsureDirectory(outputDir))
    {
        std::cerr << "Failed to create output directory: " << outputDir << "\n";
        std::fclose(file);
        return 1;
    }

    if (header.Unknown > 0)
    {
        std::vector<char> unknownBuf(header.Unknown);
        if (!ReadBytes(file, unknownBuf.data(), unknownBuf.size()))
        {
            std::cerr << "Failed to read unknown header bytes\n";
            std::fclose(file);
            return 1;
        }

        const std::string unknownPath = JoinPath(outputDir, "header_unknown.bin");
        if (!WriteBinaryFile(unknownPath, unknownBuf.data(), unknownBuf.size()))
        {
            std::cerr << "Failed to write " << unknownPath << "\n";
            std::fclose(file);
            return 1;
        }
    }

    const std::string manifestPath = JoinPath(outputDir, "manifest.txt");
    std::ofstream manifest(manifestPath.c_str());
    if (!manifest)
    {
        std::cerr << "Failed to create manifest: " << manifestPath << "\n";
        std::fclose(file);
        return 1;
    }

    manifest << "app_name=" << CleanFixedString(header.AppName, sizeof(header.AppName)) << "\n";
    manifest << "description=" << CleanFixedString(header.Description, sizeof(header.Description)) << "\n";
    manifest << "file_size=" << header.FileSize << "\n";
    manifest << "group_count=" << header.NumberOfGroups << "\n";
    manifest << "body_size=" << header.SizeOfBody << "\n";
    manifest << "full_tilt_mode=" << (fullTiltMode ? "true" : "false") << "\n";
    manifest << "export_bmp=" << (exportBmp ? "true" : "false") << "\n";
    manifest << "\n";

    std::vector<uint8_t> activePalette;

    for (uint32_t groupIndex = 0; groupIndex < header.NumberOfGroups; ++groupIndex)
    {
        uint8_t entryCount = 0;
        if (!ReadValue(file, entryCount))
        {
            std::cerr << "Unexpected EOF while reading group " << groupIndex << "\n";
            std::fclose(file);
            return 1;
        }

        std::ostringstream groupDirName;
        groupDirName << "group_" << std::setfill('0') << std::setw(4) << groupIndex;
        const std::string groupDir = JoinPath(outputDir, groupDirName.str());
        if (!EnsureDirectory(groupDir))
        {
            std::cerr << "Failed to create group directory: " << groupDir << "\n";
            std::fclose(file);
            return 1;
        }

        std::string groupName;
        manifest << "[group " << groupIndex << "] entry_count=" << static_cast<unsigned>(entryCount) << "\n";

        for (uint32_t entryIndex = 0; entryIndex < entryCount; ++entryIndex)
        {
            uint8_t rawType = 0;
            if (!ReadValue(file, rawType))
            {
                std::cerr << "Unexpected EOF while reading entry type\n";
                std::fclose(file);
                return 1;
            }

            if (rawType >= sizeof(kFieldSize) / sizeof(kFieldSize[0]))
            {
                std::cerr << "Unknown field type id " << static_cast<unsigned>(rawType)
                          << " in group " << groupIndex << "\n";
                std::fclose(file);
                return 1;
            }

            const FieldTypes fieldType = static_cast<FieldTypes>(rawType);
            int fieldSize = kFieldSize[rawType];
            if (fieldSize < 0)
            {
                uint32_t varSize = 0;
                if (!ReadValue(file, varSize))
                {
                    std::cerr << "Unexpected EOF while reading variable field size\n";
                    std::fclose(file);
                    return 1;
                }
                fieldSize = static_cast<int>(varSize);
            }

            std::ostringstream entryBase;
            entryBase << "entry_" << std::setfill('0') << std::setw(3) << entryIndex
                      << "_type" << static_cast<unsigned>(rawType)
                      << "_" << FieldTypeName(fieldType);
            const std::string entryPrefix = JoinPath(groupDir, entryBase.str());

            manifest << "entry=" << entryIndex << ", type=" << static_cast<unsigned>(rawType)
                     << ", name=" << FieldTypeName(fieldType)
                     << ", size=" << fieldSize << "\n";

            if (fieldType == FieldTypes::Bitmap8bit)
            {
                dat8BitBmpHeader bmpHeader{};
                if (!ReadValue(file, bmpHeader))
                {
                    std::cerr << "Failed to read bitmap8 header\n";
                    std::fclose(file);
                    return 1;
                }

                const int payloadSize = fieldSize - static_cast<int>(sizeof(dat8BitBmpHeader));
                if (payloadSize < 0)
                {
                    std::cerr << "Invalid bitmap8 size in group " << groupIndex << "\n";
                    std::fclose(file);
                    return 1;
                }

                std::vector<char> payload(static_cast<size_t>(payloadSize));
                if (payloadSize > 0 && !ReadBytes(file, payload.data(), payload.size()))
                {
                    std::cerr << "Failed to read bitmap8 payload\n";
                    std::fclose(file);
                    return 1;
                }

                std::ofstream meta((entryPrefix + ".meta.txt").c_str());
                meta << "resolution=" << static_cast<unsigned>(bmpHeader.Resolution) << "\n";
                meta << "width=" << bmpHeader.Width << "\n";
                meta << "height=" << bmpHeader.Height << "\n";
                meta << "x=" << bmpHeader.XPosition << "\n";
                meta << "y=" << bmpHeader.YPosition << "\n";
                meta << "size=" << bmpHeader.Size << "\n";
                meta << "flags=" << static_cast<unsigned>(bmpHeader.Flags) << "\n";

                if (!WriteBinaryFile(entryPrefix + ".bin", payload.data(), payload.size()))
                {
                    std::cerr << "Failed to write bitmap8 payload\n";
                    std::fclose(file);
                    return 1;
                }

                if (exportBmp)
                {
                    if (!bmpHeader.IsFlagSet(bmp8Flags::Spliced))
                    {
                        if (!WriteIndexedBmp(entryPrefix + ".bmp", bmpHeader.Width, bmpHeader.Height, payload, activePalette))
                        {
                            std::cerr << "Failed to write bitmap8 BMP\n";
                            std::fclose(file);
                            return 1;
                        }
                    }
                    else
                    {
                        std::ofstream note((entryPrefix + ".bmp.note.txt").c_str());
                        note << "BMP export skipped: spliced bitmap encoding requires decode pass.";
                    }
                }
            }
            else if (fieldType == FieldTypes::Bitmap16bit)
            {
                uint8_t zMapResolution = 0;
                if (fullTiltMode)
                {
                    if (!ReadValue(file, zMapResolution))
                    {
                        std::cerr << "Failed to read Full Tilt zmap resolution byte\n";
                        std::fclose(file);
                        return 1;
                    }
                    fieldSize -= 1;
                }

                dat16BitBmpHeader zMapHeader{};
                if (!ReadValue(file, zMapHeader))
                {
                    std::cerr << "Failed to read bitmap16 header\n";
                    std::fclose(file);
                    return 1;
                }

                const int payloadSize = fieldSize - static_cast<int>(sizeof(dat16BitBmpHeader));
                if (payloadSize < 0)
                {
                    std::cerr << "Invalid bitmap16 size in group " << groupIndex << "\n";
                    std::fclose(file);
                    return 1;
                }

                std::vector<char> payload(static_cast<size_t>(payloadSize));
                if (payloadSize > 0 && !ReadBytes(file, payload.data(), payload.size()))
                {
                    std::cerr << "Failed to read bitmap16 payload\n";
                    std::fclose(file);
                    return 1;
                }

                std::ofstream meta((entryPrefix + ".meta.txt").c_str());
                meta << "resolution=" << static_cast<unsigned>(zMapResolution) << "\n";
                meta << "width=" << zMapHeader.Width << "\n";
                meta << "height=" << zMapHeader.Height << "\n";
                meta << "stride=" << zMapHeader.Stride << "\n";
                meta << "unknown0=" << zMapHeader.Unknown0 << "\n";
                meta << "unknown1_0=" << zMapHeader.Unknown1_0 << "\n";
                meta << "unknown1_1=" << zMapHeader.Unknown1_1 << "\n";

                if (!WriteBinaryFile(entryPrefix + ".bin", payload.data(), payload.size()))
                {
                    std::cerr << "Failed to write bitmap16 payload\n";
                    std::fclose(file);
                    return 1;
                }
            }
            else
            {
                std::vector<char> data(static_cast<size_t>(fieldSize));
                if (fieldSize > 0 && !ReadBytes(file, data.data(), data.size()))
                {
                    std::cerr << "Failed to read field data\n";
                    std::fclose(file);
                    return 1;
                }

                if (fieldType == FieldTypes::GroupName || fieldType == FieldTypes::String)
                {
                    const std::string text = CleanCString(data);
                    std::ofstream outText((entryPrefix + ".txt").c_str());
                    outText << text;

                    if (fieldType == FieldTypes::GroupName)
                    {
                        groupName = text;
                    }
                }
                else if (fieldType == FieldTypes::ShortValue || fieldType == FieldTypes::ShortArray)
                {
                    if (!WriteBinaryFile(entryPrefix + ".bin", data.data(), data.size()))
                    {
                        std::cerr << "Failed to write short payload\n";
                        std::fclose(file);
                        return 1;
                    }

                    std::ofstream outText((entryPrefix + ".i16.txt").c_str());
                    const size_t count = data.size() / sizeof(int16_t);
                    for (size_t i = 0; i < count; ++i)
                    {
                        int16_t val = 0;
                        std::memcpy(&val, data.data() + i * sizeof(int16_t), sizeof(int16_t));
                        outText << val << "\n";
                    }
                }
                else if (fieldType == FieldTypes::FloatArray)
                {
                    if (!WriteBinaryFile(entryPrefix + ".bin", data.data(), data.size()))
                    {
                        std::cerr << "Failed to write float payload\n";
                        std::fclose(file);
                        return 1;
                    }

                    std::ofstream outText((entryPrefix + ".f32.txt").c_str());
                    const size_t count = data.size() / sizeof(float);
                    for (size_t i = 0; i < count; ++i)
                    {
                        float val = 0.0f;
                        std::memcpy(&val, data.data() + i * sizeof(float), sizeof(float));
                        outText << val << "\n";
                    }
                }
                else if (fieldType == FieldTypes::Palette && data.size() % 4 == 0)
                {
                    activePalette.assign(reinterpret_cast<const uint8_t *>(data.data()),
                                         reinterpret_cast<const uint8_t *>(data.data()) + data.size());

                    if (!WriteBinaryFile(entryPrefix + ".rgba.bin", data.data(), data.size()))
                    {
                        std::cerr << "Failed to write palette payload\n";
                        std::fclose(file);
                        return 1;
                    }

                    std::ofstream outText((entryPrefix + ".rgba.txt").c_str());
                    const size_t count = data.size() / 4;
                    for (size_t i = 0; i < count; ++i)
                    {
                        const unsigned char b = static_cast<unsigned char>(data[i * 4 + 0]);
                        const unsigned char g = static_cast<unsigned char>(data[i * 4 + 1]);
                        const unsigned char r = static_cast<unsigned char>(data[i * 4 + 2]);
                        const unsigned char a = static_cast<unsigned char>(data[i * 4 + 3]);
                        outText << i << ": r=" << static_cast<unsigned>(r)
                                << ", g=" << static_cast<unsigned>(g)
                                << ", b=" << static_cast<unsigned>(b)
                                << ", a=" << static_cast<unsigned>(a) << "\n";
                    }
                }
                else
                {
                    if (!WriteBinaryFile(entryPrefix + ".bin", data.data(), data.size()))
                    {
                        std::cerr << "Failed to write raw payload\n";
                        std::fclose(file);
                        return 1;
                    }
                }
            }
        }

        if (!groupName.empty())
        {
            manifest << "group_name=" << groupName << "\n";
            std::ofstream groupNameFile(JoinPath(groupDir, "group_name.txt").c_str());
            groupNameFile << groupName;
        }

        manifest << "\n";
    }

    std::fclose(file);
    std::cout << "Extraction complete. Output: " << outputDir << "\n";
    return 0;
}

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        PrintUsage();
        return 1;
    }

    const std::string command = argv[1];
    if (command == "extract")
    {
        if (argc < 3)
        {
            PrintUsage();
            return 1;
        }

        const std::string inputPath = argv[2];
        std::string outputDir = "extract_out";
        bool fullTiltMode = false;
        bool exportBmp = false;

        for (int i = 3; i < argc; ++i)
        {
            const std::string arg = argv[i];
            if (arg == "--full-tilt")
            {
                fullTiltMode = true;
            }
            else if (arg == "--bmp")
            {
                exportBmp = true;
            }
            else
            {
                outputDir = arg;
            }
        }

        return ExtractDat(inputPath, outputDir, fullTiltMode, exportBmp);
    }

    PrintUsage();
    return 1;
}
