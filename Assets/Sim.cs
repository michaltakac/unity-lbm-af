using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class Sim : MonoBehaviour
{
	public static int domain_width = 300;
    public static int domain_height = 100;

    // Physical parameters.
    public static float initial_density = 1.0f;

    // x location of the cylinder
    public static uint obstacle_x = (uint)domain_width / 5 + 1;
    // y location of the cylinder
    public static uint obstacle_y = (uint)domain_height / 2 + (uint)domain_height / 30;
    // radius of the cylinder
    public static uint obstacle_r = (uint)domain_height / 10 + 1;


	public float inflowDensity = 1.0f;
    public float inflowSpeed = 0.1f;
	public float reynoldsNumber = 220.0f;

    public static float inflow_density;
	public static float inflow_speed;
    // Reynolds number
    public static float re;
    // Kinematic viscosity
    public static float nu;
    // Relaxation time
    public static float tau;
    // Relaxation parameter
	public float omega;
    public static float omega_static;

    private static byte[] buffer;
    private Int32 size;
    private Texture2D image;


    void Start()
    {
		// Initialize
		inflow_speed = inflowSpeed;
		inflow_density = inflowDensity;
		re = reynoldsNumber;
		nu = inflow_speed * 2.0f * obstacle_r / re;
    	tau = 3.0f * nu + 0.5f;
		omega_static = 1.0f / tau;

		// ---------
        size = domain_width * domain_height * 4;
        buffer = new byte[size];
        image = new Texture2D(domain_width, domain_height, TextureFormat.RGBA32, false);
        GetComponent<Renderer>().material.mainTexture = image;

        bool isSimReady = LBMAF.InitSimulation(
            (uint)domain_width, (uint)domain_height, initial_density, inflow_speed,
            omega_static, obstacle_x, obstacle_y, obstacle_r);
        if (!isSimReady) return;

        Debug.Log("Simulation initialized.");
        StartCoroutine("GetData");
    }


    public IEnumerator GetData()
    {
        while (true)
        {
			inflow_speed = inflowSpeed;
			inflow_density = inflowDensity;
			re = reynoldsNumber;
			nu = inflow_speed * 2.0f * obstacle_r / re;
			tau = 3.0f * nu + 0.5f;
			omega = omega_static = 1.0f / tau;

            LBMAF.SimulateNextIteration(inflow_density, inflow_speed, omega_static);
            LBMAF.GetSimData();
            LBMAF.CopyResultsToBuffer(buffer, size);
            image.LoadRawTextureData(buffer);
            image.Apply();
            yield return null;
        }
    }

    void Update()
    {

    }

    void OnDestroy()
    {
        image = null;
		buffer = null;
		GetComponent<Renderer>().material.mainTexture = null;
		// DisposeSim();
		Debug.Log("Destroyed the simulation resources.");
    }
}
