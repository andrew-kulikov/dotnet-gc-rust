using System;

namespace GraphFixtures;

public static class Log
{
    public static void Info(string message)
    {
        Console.WriteLine($"dotnet: {message}");
    }
}